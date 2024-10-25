use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use line_span::LineSpans;
use lspower::jsonrpc;
use lspower::lsp::*;
use lspower::{Client, LanguageServer, LspService, Server};
use mers_lib::data::Type;
use mers_lib::errors::CheckError;
use mers_lib::parsing::SourceFrom;
use mers_lib::prelude_extend_config::Config;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, TextDocument>>,
}

#[derive(Debug)]
struct TextDocument {
    source: String,
    file_path: Option<PathBuf>,
    srca: Option<Arc<mers_lib::prelude_compile::Source>>,
    infos: Option<(
        mers_lib::program::parsed::Info,
        mers_lib::program::run::Info,
        mers_lib::program::run::CheckInfo,
    )>,
    parsed: Option<Result<Box<dyn mers_lib::program::parsed::MersStatement>, CheckError>>,
    compiled: Option<Result<Box<dyn mers_lib::program::run::MersStatement>, CheckError>>,
    checked: Option<Result<mers_lib::data::Type, CheckError>>,
}
impl TextDocument {
    pub fn changed(&mut self) {
        self.srca = None;
        self.infos = None;
        self.parsed = None;
        self.compiled = None;
        self.checked = None;
    }
    pub fn srca(&mut self) -> &Arc<mers_lib::prelude_compile::Source> {
        if self.srca.is_none() {
            self.srca = Some(Arc::new(mers_lib::prelude_compile::Source::new(
                self.file_path
                    .clone()
                    .map(|v| SourceFrom::File(v))
                    .unwrap_or(SourceFrom::Unspecified),
                self.source.clone(),
            )));
        }
        self.srca.as_ref().unwrap()
    }
    pub fn parsed(
        &mut self,
        force: bool,
    ) -> (
        &Result<Box<dyn mers_lib::program::parsed::MersStatement>, CheckError>,
        &mut (
            mers_lib::program::parsed::Info,
            mers_lib::program::run::Info,
            mers_lib::program::run::CheckInfo,
        ),
    ) {
        if force || self.parsed.is_none() {
            self.parsed = Some(mers_lib::prelude_compile::parse(
                &mut mers_lib::prelude_compile::Source::clone(self.srca()),
                self.srca(),
            ))
        }
        (
            self.parsed.as_ref().unwrap(),
            self.infos.get_or_insert_with(gen_infos),
        )
    }
    pub fn compiled(
        &mut self,
        force: bool,
    ) -> (
        &Result<Box<dyn mers_lib::program::run::MersStatement>, CheckError>,
        &mut (
            mers_lib::program::parsed::Info,
            mers_lib::program::run::Info,
            mers_lib::program::run::CheckInfo,
        ),
    ) {
        if force || self.compiled.is_none() {
            let (parsed, infos) = self.parsed(false);
            self.compiled = Some(
                parsed
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|v| mers_lib::prelude_compile::compile_mut(&**v, &mut infos.0)),
            );
        }
        (
            self.compiled.as_ref().unwrap(),
            self.infos.get_or_insert_with(gen_infos),
        )
    }
    pub fn checked(
        &mut self,
        force: bool,
    ) -> (
        &Result<mers_lib::data::Type, CheckError>,
        &mut (
            mers_lib::program::parsed::Info,
            mers_lib::program::run::Info,
            mers_lib::program::run::CheckInfo,
        ),
    ) {
        if force || self.checked.is_none() {
            let (compiled, infos) = self.compiled(false);
            self.checked = Some(
                compiled
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|v| mers_lib::prelude_compile::check_mut(&**v, &mut infos.2)),
            );
        }
        (
            self.checked.as_ref().unwrap(),
            self.infos.get_or_insert_with(gen_infos),
        )
    }
}
fn gen_infos() -> (
    mers_lib::program::parsed::Info,
    mers_lib::program::run::Info,
    mers_lib::program::run::CheckInfo,
) {
    Config::new().bundle_std().infos()
}

#[lspower::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let mut init = InitializeResult::default();
        init.capabilities.text_document_sync =
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL));
        init.capabilities.completion_provider = Some(CompletionOptions::default());
        init.capabilities.hover_provider = Some(HoverProviderCapability::Simple(true));
        Ok(init)
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let fp = params.text_document.uri.to_file_path();
        self.documents.lock().unwrap().insert(
            params.text_document.uri,
            TextDocument {
                source: params.text_document.text,
                file_path: fp.ok(),
                srca: None,
                infos: None,
                parsed: None,
                compiled: None,
                checked: None,
            },
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut last_text = None;
        for change in params.content_changes {
            if change.range.is_none() {
                last_text = Some(change.text);
            }
        }
        if let Some(new_text) = last_text {
            if let Some(d) = self
                .documents
                .lock()
                .unwrap()
                .get_mut(&params.text_document.uri)
            {
                d.source = new_text;
                d.changed();
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents
            .lock()
            .unwrap()
            .remove(&params.text_document.uri);
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        if let Some(doc) = self
            .documents
            .lock()
            .unwrap()
            .get_mut(&params.text_document_position.text_document.uri)
        {
            let byte_pos = doc_byte_pos(&doc.source, params.text_document_position.position);
            let byte_pos_in_src = doc.srca().pos_from_og(byte_pos, false);
            if let (Ok(parsed), (i1, _, _)) = doc.parsed(false) {
                i1.global.enable_hooks = true;
                let mut save_info_at = i1.global.save_info_at.lock().unwrap();
                let my_saved_info_index = save_info_at.len();
                save_info_at.push((vec![], byte_pos_in_src, 0));
                drop((save_info_at, parsed));
                let variable_types = if let (Ok(_), (_, _, i3)) = doc.compiled(true) {
                    i3.global.enable_hooks = true;
                    let mut save_info_at = i3.global.save_info_at.lock().unwrap();
                    let my_saved_info_index = save_info_at.len();
                    save_info_at.push((vec![], byte_pos_in_src, 0));
                    drop(save_info_at);
                    _ = doc.checked(true);
                    let (_, _, i3) = doc.infos.get_or_insert_with(gen_infos);
                    let save_info_at = i3.global.save_info_at.lock().unwrap();
                    let my_saved_info = &save_info_at[my_saved_info_index].0;
                    let mut variable_types: HashMap<(usize, usize), Type> = HashMap::new();
                    for (_range, info, _err) in my_saved_info {
                        for (is, scope) in info.scopes.iter().enumerate() {
                            for (iv, var) in scope.vars.iter().enumerate() {
                                let key = (is, iv);
                                if let Some(t) = variable_types.get_mut(&key) {
                                    t.add_all(var);
                                } else {
                                    variable_types.insert(key, var.clone());
                                }
                            }
                        }
                    }
                    drop(save_info_at);
                    Some(variable_types)
                } else {
                    None
                };
                let (i1, _, i3) = doc.infos.get_or_insert_with(gen_infos);
                let save_info_at = i1.global.save_info_at.lock().unwrap();
                let result = &save_info_at[my_saved_info_index].0;
                let mut variables = HashMap::new();
                for (_range, a, _err) in result {
                    for (depth, scope) in a.scopes.iter().enumerate() {
                        for (var, v) in scope.vars.iter() {
                            variables.insert(var, (depth, *v));
                        }
                    }
                }
                let mut variables = variables.into_iter().collect::<Vec<_>>();
                // sort by depth (rev), then name -> local variables first, globals later, within the same scope A->Z
                variables.sort_unstable_by(|a, b| b.1 .0.cmp(&a.1 .0).then_with(|| a.0.cmp(&b.0)));
                let items = variables
                    .into_iter()
                    .map(|v| {
                        CompletionItem::new_simple(
                            v.0.clone(),
                            if let Some(typ) =
                                variable_types.as_ref().and_then(|types| types.get(&v.1 .1))
                            {
                                format!("var: `{}`", typ.with_info(i3))
                            } else {
                                format!("variable")
                            },
                        )
                    })
                    .collect();
                drop(save_info_at);
                return Ok(Some(CompletionResponse::Array(items)));
            }
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> crate::jsonrpc::Result<Option<Hover>> {
        let this_doc = params.text_document_position_params.text_document.uri;
        Ok(
            if let Some(doc) = self.documents.lock().unwrap().get_mut(&this_doc) {
                let pos = params.text_document_position_params.position;
                let byte_pos = doc_byte_pos(&doc.source, pos);
                Some(match doc.compiled(false) {
                    (Ok(compiled), infos) => {
                        let file = Arc::clone(compiled.source_range().in_file());
                        let pos = file.pos_from_og(byte_pos, false);
                        let (_, _, i3) = infos;
                        i3.global.enable_hooks = true;
                        let save_info_at = Arc::new(Mutex::new(vec![(vec![], pos, 0)]));
                        i3.global.save_info_at = Arc::clone(&save_info_at);
                        _ = doc.checked(true);
                        let (_, _, i3) = doc.infos.get_or_insert_with(gen_infos);
                        let hook_res = &save_info_at.lock().unwrap()[0];
                        Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "Local Types:{}",
                                    hook_res
                                        .0
                                        .iter()
                                        .map(|(_, _, t)| match t {
                                            Ok(t) => format!("\n- {}", t.with_info(&i3)),
                                            Err(e) => format!("\n- {}", e.display_notheme()),
                                        })
                                        .collect::<String>()
                                ),
                            }),
                            range: None,
                        }
                    }
                    (Err(e), _) => Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::PlainText,
                            value: format!("{}", e.display_notheme()),
                        }),
                        range: None,
                    },
                })
            } else {
                None
            },
        )
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(|client| Backend {
        client,
        documents: Default::default(),
    });
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}

fn doc_byte_pos(src: &str, pos: Position) -> usize {
    let line_start = src
        .line_spans()
        .take(pos.line as _)
        .map(|l| l.as_str_with_ending().len())
        .sum::<usize>();
    let line_pos = src
        .chars()
        .take(pos.character as _)
        .map(|c| c.len_utf8())
        .sum::<usize>();
    line_start + line_pos
}
