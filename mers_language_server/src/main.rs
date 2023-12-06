use std::{collections::HashMap, sync::Arc};

use line_span::LineSpanExt;
use mers_lib::{errors::CheckError, prelude_compile::Source};
use tokio::sync::Mutex;
use tower_lsp::{
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, DidChangeTextDocumentParams,
        DidOpenTextDocumentParams, DocumentChanges, Hover, HoverContents, HoverParams,
        HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
        MarkedString, MessageType, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
        ServerCapabilities, TextDocumentEdit, TextDocumentItem, TextDocumentSyncCapability,
        TextDocumentSyncKind, TextEdit, WorkDoneProgressOptions, WorkspaceEdit,
    },
    Client, LanguageServer, LspService, Server,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}

struct Backend {
    client: Client,
    current_document: Arc<Mutex<Option<tower_lsp::lsp_types::Url>>>,
    documents: Arc<Mutex<HashMap<tower_lsp::lsp_types::Url, TextDocumentItem>>>,
    last_compiled: Arc<Mutex<Option<ParseCompileCheckResult>>>,
}
impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            current_document: Arc::new(Mutex::new(None)),
            documents: Arc::new(Mutex::new(HashMap::new())),
            last_compiled: Arc::new(Mutex::new(None)),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn did_open(&self, p: DidOpenTextDocumentParams) {
        {
            let mut md = self.current_document.lock().await;
            *md = Some(p.text_document.uri.clone());
            *self.last_compiled.lock().await = None;
        }
        self.documents
            .lock()
            .await
            .insert(p.text_document.uri.clone(), p.text_document);
    }
    async fn did_change(&self, p: DidChangeTextDocumentParams) {
        {
            let mut md = self.current_document.lock().await;
            *md = Some(p.text_document.uri.clone());
            *self.last_compiled.lock().await = None;
        }
        if let Some(document) = self.documents.lock().await.get_mut(&p.text_document.uri) {
            for change in p.content_changes {
                if let Some(_range) = change.range {
                    self.client
                        .log_message(MessageType::WARNING, "Received incremental document update")
                        .await;
                } else {
                    document.text = change.text;
                }
            }
        } else {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "Received changes for a document that was never loaded",
                )
                .await;
        }
    }

    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        // TODO: PositionEncodingKind, set char to 0-based *byte*-index
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::new("Test")]),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        resolve_provider: None,
                    },
                )),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn hover(&self, p: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        {
            let mut md = self.current_document.lock().await;
            *md = Some(p.text_document_position_params.text_document.uri.clone());
            *self.last_compiled.lock().await = None;
        }
        let byte_pos = {
            match self.current_document.lock().await.as_ref() {
                Some(uri) => {
                    let doc = self.documents.lock().await;
                    let doc = doc.get(uri);
                    let doc = doc.map(|doc| doc.text.as_str()).unwrap_or("");
                    let pos_in_og = get_byte_pos_in_og(
                        doc,
                        p.text_document_position_params.position.line as _,
                        p.text_document_position_params.position.character as _,
                    );
                    Some(
                        mers_lib::prelude_compile::Source::new_from_string(doc.to_owned())
                            .pos_from_og(pos_in_og, false),
                    )
                }
                None => None,
            }
        };
        let mut infos_at_cursor_hook_index = None;
        let pcc = self
            .parse_compile_check(|i1, i3| {
                i1.global.enable_hooks = true;
                i3.global.enable_hooks = true;
                if let Some(byte_pos) = byte_pos {
                    let mut i1sia = i1.global.save_info_at.lock().unwrap();
                    let mut i3sia = i3.global.save_info_at.lock().unwrap();
                    infos_at_cursor_hook_index = Some((i1sia.len(), i3sia.len()));
                    i1sia.push((vec![], byte_pos, 2)); // 2 -> ignore outer scope
                    i3sia.push((vec![], byte_pos, 2)); // -> global error doesn't show up in "local" section
                }
            })
            .await;
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(match pcc.as_ref().unwrap() {
                ParseCompileCheckResult::NoMainDocument => {
                    format!("# No main document\n\n(open at least one file!)")
                }
                ParseCompileCheckResult::MainDocumentNotFound => {
                    format!("# Main document not found\n\n(probably a bug)")
                }
                ParseCompileCheckResult::ErrorWhileParsing(_, _, e) => {
                    format!("# Error (can't parse):\n```\n{e}```")
                }
                ParseCompileCheckResult::ErrorWhileCompiling(_, _, e, _, info1) => {
                    let i1sia = info1.global.save_info_at.lock().unwrap();
                    format!(
                        "# Error (can't compile):{}\n## Global error:\n\n```\n{e}```",
                        if let Some(i1) = infos_at_cursor_hook_index.map(|(i, _)| &i1sia[i].0) {
                            format!(
                                "\n\n## Local errors:{}\n---\n",
                                i1.iter()
                                    .filter_map(|(src_range, _info, res)| {
                                        let src_snippet = src_range.in_file().src()
                                            [src_range.start().pos()..src_range.end().pos()]
                                            .replace("\n", "  ")
                                            .replace("\r", "  ");
                                        match res {
                                            Ok(()) => None,
                                            Err(e) => {
                                                Some(format!("\n- `{src_snippet}`\n```\n{e}```\n"))
                                            }
                                        }
                                    })
                                    .collect::<String>()
                            )
                        } else {
                            format!("")
                        }
                    )
                }
                ParseCompileCheckResult::DoesntPassChecks(_, _, e, _, _, _, info3) => {
                    let i3sia = info3.global.save_info_at.lock().unwrap();
                    format!(
                        "# Error (doesn't pass checks):{}\n## Global error:\n\n```\n{e}```",
                        if let Some(i3) = infos_at_cursor_hook_index.map(|(_, i)| &i3sia[i].0) {
                            format!(
                                "\n\n## Local types:{}\n---\n",
                                i3.iter()
                                    .map(|(src_range, _info, res)| {
                                        let src_snippet = src_range.in_file().src()
                                            [src_range.start().pos()..src_range.end().pos()]
                                            .replace("\n", "  ")
                                            .replace("\r", "  ");
                                        match res {
                                            Ok(local_type) => {
                                                format!("\n- `{src_snippet}` :: `{local_type}`")
                                            }
                                            Err(e) => {
                                                format!(
                                                    "\n- `{src_snippet}` :: ! Err !\n```\n{e}```\n"
                                                )
                                            }
                                        }
                                    })
                                    .collect::<String>()
                            )
                        } else {
                            format!("")
                        }
                    )
                }
                ParseCompileCheckResult::Checked(_, _, out_type, _, _, _, info3) => {
                    let i3sia = info3.global.save_info_at.lock().unwrap();
                    if let Some(i3) = infos_at_cursor_hook_index.map(|(_, i)| &i3sia[i].0) {
                        format!(
                            "## Local types:{}",
                            i3.iter()
                                .map(|(src_range, _info, res)| {
                                    let src_snippet = src_range.in_file().src()
                                        [src_range.start().pos()..src_range.end().pos()]
                                        .replace("\n", "  ")
                                        .replace("\r", "  ");
                                    match res {
                                        Ok(local_type) => {
                                            format!("\n- `{src_snippet}` :: `{local_type}`")
                                        }
                                        Err(e) => {
                                            format!("\n- `{src_snippet}` :: ! Err !\n```\n{e}```\n")
                                        }
                                    }
                                })
                                .collect::<String>()
                        )
                    } else {
                        format!("Program's return type: `{out_type}`")
                    }
                }
            })),
            range: None,
        }))
    }

    async fn code_action(
        &self,
        p: CodeActionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
        {
            let mut md = self.current_document.lock().await;
            if !md.as_ref().is_some_and(|md| *md == p.text_document.uri) {
                *md = Some(p.text_document.uri.clone());
                *self.last_compiled.lock().await = None;
            }
        }
        Ok(Some(
            if let Some(doc) = self.documents.lock().await.get(&p.text_document.uri) {
                let mut src = mers_lib::prelude_compile::Source::new_from_string(doc.text.clone());
                let srca = Arc::new(src.clone());
                match mers_lib::prelude_compile::parse(&mut src, &srca) {
                    Err(_) => return Ok(None),
                    Ok(parsed) => {
                        let pos_start = srca.pos_from_og(
                            get_byte_pos_in_og(
                                srca.src_og(),
                                p.range.start.line as _,
                                p.range.start.character as _,
                            ),
                            false,
                        );
                        let pos_end = srca.pos_from_og(
                            get_byte_pos_in_og(
                                srca.src_og(),
                                p.range.end.line as _,
                                p.range.end.character as _,
                            ),
                            true,
                        );
                        let mut statements = vec![];
                        iter_over_statements(parsed.as_ref(), &mut |s| {
                            // Wrap in `( )`
                            if s.source_range().start().pos() <= pos_start
                                && pos_end <= s.source_range().end().pos()
                            {
                                statements.push(s);
                                s.inner_statements()
                            } else {
                                vec![]
                            }
                        });
                        let statements = statements
                            .into_iter()
                            .rev()
                            .take(3)
                            .map(|s| {
                                (
                                    s.source_range()
                                        .in_file()
                                        .pos_in_og(s.source_range().start().pos(), true),
                                    s.source_range()
                                        .in_file()
                                        .pos_in_og(s.source_range().end().pos(), false),
                                    s,
                                )
                            })
                            .collect::<Vec<_>>();
                        let mut actions = vec![];

                        // Wrap in `( )`
                        for (og_start, og_end, statement) in &statements {
                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!(
                                    "Wrap `{}` in `( )`",
                                    srca.src()[statement.source_range().start().pos()
                                        ..statement.source_range().end().pos()]
                                        .trim()
                                        .replace("\n", "  ")
                                        .replace("\r", "  ")
                                ),
                                edit: Some(WorkspaceEdit {
                                    changes: None,
                                    document_changes: Some(DocumentChanges::Edits(vec![
                                        TextDocumentEdit {
                                            text_document:
                                                OptionalVersionedTextDocumentIdentifier {
                                                    uri: p.text_document.uri.clone(),
                                                    version: None,
                                                },
                                            edits: vec![OneOf::Left(TextEdit {
                                                range: tower_lsp::lsp_types::Range {
                                                    start: get_lsp_pos_in_og(
                                                        statement.source_range().in_file().src_og(),
                                                        *og_start,
                                                    ),
                                                    end: get_lsp_pos_in_og(
                                                        statement.source_range().in_file().src_og(),
                                                        *og_end,
                                                    ),
                                                },
                                                new_text: format!(
                                                    "({})",
                                                    &statement.source_range().in_file().src_og()
                                                        [*og_start..*og_end]
                                                ),
                                            })],
                                        },
                                    ])),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }));
                        }

                        actions
                    }
                }
            } else {
                return Ok(None);
            },
        ))
    }
}

/// Recursively iterate over statements.
fn iter_over_statements<'b, S>(root: S, for_each: &'b mut impl FnMut(S) -> Vec<S>) {
    for i in for_each(root) {
        iter_over_statements(i, for_each);
    }
}

fn get_byte_pos_in_og(og: &str, line: usize, character: usize) -> usize {
    let line = og.line_spans().skip(line).next();
    if let Some(line) = line {
        line.start() + character
    } else {
        og.len()
    }
}
fn get_lsp_pos_in_og(og: &str, pos: usize) -> Position {
    let mut fallback = Position {
        line: 0,
        character: 0,
    };
    for (line_nr, line) in og.line_spans().enumerate() {
        fallback = Position {
            line: line_nr as _,
            character: line.as_str().len() as _,
        };
        if pos <= line.end() {
            return Position {
                line: line_nr as _,
                character: (pos.saturating_sub(line.start())).min(line.as_str().len()) as _,
            };
        }
    }
    fallback
}

enum ParseCompileCheckResult {
    NoMainDocument,
    MainDocumentNotFound,
    ErrorWhileParsing(Source, Arc<Source>, CheckError),
    ErrorWhileCompiling(
        Source,
        Arc<Source>,
        CheckError,
        Box<dyn mers_lib::prelude_compile::ParsedMersStatement>,
        mers_lib::program::parsed::Info,
    ),
    DoesntPassChecks(
        Source,
        Arc<Source>,
        CheckError,
        Box<dyn mers_lib::prelude_compile::ParsedMersStatement>,
        Box<dyn mers_lib::prelude_compile::RunMersStatement>,
        mers_lib::program::parsed::Info,
        mers_lib::program::run::CheckInfo,
    ),
    Checked(
        Source,
        Arc<Source>,
        mers_lib::data::Type,
        Box<dyn mers_lib::prelude_compile::ParsedMersStatement>,
        Box<dyn mers_lib::prelude_compile::RunMersStatement>,
        mers_lib::program::parsed::Info,
        mers_lib::program::run::CheckInfo,
    ),
}
impl Backend {
    /// is guaranteed to return `MutexGuard<Some(_)>`.
    async fn parse_compile_check(
        &self,
        func_modify_infos: impl FnOnce(
            &mut mers_lib::program::parsed::Info,
            &mut mers_lib::program::run::CheckInfo,
        ),
    ) -> tokio::sync::MutexGuard<'_, Option<ParseCompileCheckResult>> {
        let mut last_compiled = self.last_compiled.lock().await;
        if last_compiled.is_none() {
            *last_compiled = Some(
                if let Some(source) = self.current_document.lock().await.clone() {
                    if let Some(document) = self.documents.lock().await.get(&source) {
                        let src_from = match document.uri.to_file_path() {
                            Ok(path) => mers_lib::parsing::SourceFrom::File(path),
                            Err(_) => mers_lib::parsing::SourceFrom::Unspecified,
                        };
                        let mut src =
                            mers_lib::prelude_compile::Source::new(src_from, document.text.clone());
                        let srca = Arc::new(src.clone());
                        match mers_lib::parsing::parse(&mut src, &srca) {
                            Err(e) => ParseCompileCheckResult::ErrorWhileParsing(src, srca, e),
                            Ok(parsed) => {
                                let (mut i1, _, mut i3) = mers_lib::prelude_compile::Config::new()
                                    .bundle_std()
                                    .infos();
                                func_modify_infos(&mut i1, &mut i3);
                                match parsed.compile(
                                    &mut i1,
                                    mers_lib::prelude_compile::CompInfo::default(),
                                ) {
                                    Err(e) => ParseCompileCheckResult::ErrorWhileCompiling(
                                        src, srca, e, parsed, i1,
                                    ),
                                    Ok(compiled) => match compiled.check(&mut i3, None) {
                                        Err(e) => ParseCompileCheckResult::DoesntPassChecks(
                                            src, srca, e, parsed, compiled, i1, i3,
                                        ),
                                        Ok(out_type) => ParseCompileCheckResult::Checked(
                                            src, srca, out_type, parsed, compiled, i1, i3,
                                        ),
                                    },
                                }
                            }
                        }
                    } else {
                        ParseCompileCheckResult::MainDocumentNotFound
                    }
                } else {
                    ParseCompileCheckResult::NoMainDocument
                },
            );
        }
        last_compiled
    }
}
