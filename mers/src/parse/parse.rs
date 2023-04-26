use std::{process::Command, sync::Arc};

use crate::{
    libs,
    script::{
        code_parsed::*,
        code_runnable::RScript,
        global_info::GSInfo,
        to_runnable::{self, GInfo, ToRunnableError},
        val_data::VDataEnum,
        val_type::{VSingleType, VType},
    },
};

use super::file::File;

pub enum ScriptError {
    CannotFindPathForLibrary(CannotFindPathForLibrary),
    ParseError(ParseError),
    UnableToLoadLibrary(UnableToLoadLibrary),
    ToRunnableError(ToRunnableError),
}
impl From<CannotFindPathForLibrary> for ScriptError {
    fn from(value: CannotFindPathForLibrary) -> Self {
        Self::CannotFindPathForLibrary(value)
    }
}
impl From<ParseError> for ScriptError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}
impl From<UnableToLoadLibrary> for ScriptError {
    fn from(value: UnableToLoadLibrary) -> Self {
        Self::UnableToLoadLibrary(value)
    }
}
impl From<ToRunnableError> for ScriptError {
    fn from(value: ToRunnableError) -> Self {
        Self::ToRunnableError(value)
    }
}
impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CannotFindPathForLibrary(e) => write!(f, "{e}"),
            Self::ParseError(e) => write!(f, "failed while parsing: {e}"),
            Self::UnableToLoadLibrary(e) => write!(f, "{e}"),
            Self::ToRunnableError(e) => write!(f, "failed to compile: {e}"),
        }
    }
}
pub struct ScriptErrorWithFile<'a>(&'a ScriptError, &'a File);
impl<'a> ScriptError {
    pub fn with_file(&'a self, file: &'a File) -> ScriptErrorWithFile {
        ScriptErrorWithFile(self, file)
    }
}
impl<'a> std::fmt::Display for ScriptErrorWithFile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ScriptError::CannotFindPathForLibrary(e) => write!(f, "{e}"),
            ScriptError::ParseError(e) => {
                write!(f, "failed while parsing: {}", e.with_file(self.1))
            }
            ScriptError::UnableToLoadLibrary(e) => write!(f, "{e}"),
            ScriptError::ToRunnableError(e) => write!(f, "failed to compile: {e}"),
        }
    }
}

pub const PARSE_VERSION: u64 = 0;

/// executes the 4 parse_steps in order: lib_paths => interpret => libs_load => compile
pub fn parse(file: &mut File) -> Result<RScript, ScriptError> {
    let mut ginfo = GInfo::default();
    let libs = parse_step_lib_paths(file)?;
    let func = parse_step_interpret(file)?;
    ginfo.libs = parse_step_libs_load(libs, &mut ginfo)?;

    let run = parse_step_compile(func, ginfo)?;

    Ok(run)
}

#[derive(Debug)]
pub struct CannotFindPathForLibrary(String);
impl std::error::Error for CannotFindPathForLibrary {}
impl std::fmt::Display for CannotFindPathForLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Couldn't find a path for the library with the path '{}'. Maybe set the MERS_LIB_DIR env variable?", self.0)
    }
}
pub fn parse_step_lib_paths(file: &mut File) -> Result<Vec<Command>, CannotFindPathForLibrary> {
    let mut libs = vec![];
    loop {
        file.skip_whitespaces();
        let pos = file.get_pos().clone();
        let line = file.next_line();
        if line.starts_with("lib ") {
            let path_to_executable = match libs::path::path_from_string(&line[4..], file.path()) {
                Some(v) => v,
                None => return Err(CannotFindPathForLibrary(line[4..].to_string())),
            };
            let mut cmd = Command::new(&path_to_executable);
            if let Some(parent) = path_to_executable.parent() {
                cmd.current_dir(parent.clone());
            }
            libs.push(cmd);
        } else {
            file.set_pos(pos);
            break;
        }
    }
    Ok(libs)
}

pub fn parse_step_interpret(file: &mut File) -> Result<SFunction, ParseError> {
    Ok(SFunction::new(
        vec![(
            "args".to_string(),
            VSingleType::List(VSingleType::String.into()).to(),
        )],
        parse_block_advanced(file, Some(false), true, true, false)?,
    ))
}

#[derive(Debug)]
pub struct UnableToLoadLibrary(String, crate::libs::LaunchError);
impl std::fmt::Display for UnableToLoadLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to load library {}: {}", self.0, self.1)
    }
}
pub fn parse_step_libs_load(
    lib_cmds: Vec<Command>,
    ginfo: &mut GInfo,
) -> Result<Vec<libs::Lib>, UnableToLoadLibrary> {
    let mut libs = vec![];
    for cmd in lib_cmds {
        let cmd_path = cmd.get_program().to_string_lossy().into_owned();
        match libs::Lib::launch(cmd, &mut ginfo.enum_variants) {
            Ok(lib) => {
                for (i, (func, _, _)) in lib.registered_fns.iter().enumerate() {
                    ginfo.lib_fns.insert(func.clone(), (libs.len(), i));
                }
                libs.push(lib);
            }
            Err(e) => return Err(UnableToLoadLibrary(cmd_path, e)),
        }
    }
    Ok(libs)
}

pub fn parse_step_compile(main_func: SFunction, ginfo: GInfo) -> Result<RScript, ToRunnableError> {
    to_runnable::to_runnable(main_func, ginfo)
}

pub struct ParseErrorWithFile<'a>(&'a ParseError, &'a File);
impl<'a> ParseError {
    pub fn with_file(&'a self, file: &'a File) -> ParseErrorWithFile {
        ParseErrorWithFile(self, file)
    }
}
impl<'a> std::fmt::Display for ParseErrorWithFile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_custom(f, Some(self.1))
    }
}
pub struct ParseError {
    err: ParseErrors,
    // the location of the error
    location: super::file::FilePosition,
    location_end: Option<super::file::FilePosition>,
    context: Vec<(
        String,
        Option<(super::file::FilePosition, Option<super::file::FilePosition>)>,
    )>,
    info: Option<GSInfo>,
}
impl ParseError {
    pub fn fmt_custom(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        file: Option<&super::file::File>,
    ) -> std::fmt::Result {
        self.err.fmtgs(f, self.info.as_ref(), file)?;
        writeln!(f);
        if let Some(location_end) = self.location_end {
            writeln!(f, "  from {} to {}", self.location, location_end)?;
            if let Some(file) = file {
                if self.location.current_line == location_end.current_line {
                    write!(
                        f,
                        "    {}\n    {}{} here",
                        file.get_line(self.location.current_line).unwrap(),
                        " ".repeat(self.location.current_column),
                        "^".repeat(
                            location_end
                                .current_column
                                .saturating_sub(self.location.current_column)
                                .saturating_add(1)
                        )
                    )?;
                }
            }
        } else {
            writeln!(f, "  at {}", self.location)?;
        }
        for ctx in self.context.iter() {
            writeln!(f, "  {}", ctx.0)?;
            if let Some(pos) = &ctx.1 {
                if let Some(end) = &pos.1 {
                    writeln!(f, "    from {} to {}", pos.0, end)?;
                } else {
                    writeln!(f, "    at {}", pos.0)?;
                }
            }
        }
        Ok(())
    }
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_custom(f, None)
    }
}
pub enum ParseErrors {
    StatementCannotStartWith(char),
    FoundClosingRoundBracketInSingleStatementBlockBeforeAnyStatement,
    FoundClosingCurlyBracketInSingleStatementBlockBeforeAnyStatement,
    FoundEofInBlockBeforeStatementOrClosingCurlyBracket,
    FoundEofInString,
    FoundEofInStatement,
    FoundEofInFunctionArgName,
    FoundEofInType,
    FoundEofInsteadOfType,
    InvalidType(String),
    CannotUseFixedIndexingWithThisType(VType),
    CannotWrapWithThisStatement(SStatementEnum),
    ErrorParsingFunctionArgs(Box<ParseError>),
}
impl ParseErrors {
    fn fmtgs(
        &self,
        f: &mut std::fmt::Formatter,
        info: Option<&GSInfo>,
        file: Option<&super::file::File>,
    ) -> std::fmt::Result {
        match self {
            Self::StatementCannotStartWith(ch) => {
                write!(f, "statements cannot start with the {ch} character.",)
            }
            Self::FoundClosingRoundBracketInSingleStatementBlockBeforeAnyStatement => write!(
                f,
                "closing round bracket in single-statement block before any statement"
            ),
            Self::FoundClosingCurlyBracketInSingleStatementBlockBeforeAnyStatement => write!(
                f,
                "closing curly bracket in single-statement block before any statement."
            ),
            Self::FoundEofInBlockBeforeStatementOrClosingCurlyBracket => write!(
                f,
                "found EOF in block before any statement or closing curly bracket was found."
            ),
            Self::FoundEofInString => write!(f, "found EOF in string literal."),
            Self::FoundEofInStatement => write!(f, "found EOF in statement."),
            Self::FoundEofInFunctionArgName => {
                write!(f, "found EOF in the name of the function's argument.")
            }
            Self::FoundEofInType => write!(f, "found EOF in type."),
            Self::FoundEofInsteadOfType => write!(f, "expected type, found EOF instead."),
            Self::InvalidType(name) => write!(f, "\"{name}\" is not a type."),
            Self::CannotUseFixedIndexingWithThisType(t) => {
                write!(f, "cannot use fixed-indexing with type ")?;
                t.fmtgs(f, info)?;
                write!(f, ".")
            }
            Self::CannotWrapWithThisStatement(s) => {
                write!(f, "cannot wrap with this kind of statement: ")?;
                s.fmtgs(f, info)?;
                write!(f, ".")
            }
            Self::ErrorParsingFunctionArgs(parse_error) => {
                write!(f, "error parsing function args: ")?;
                parse_error.fmt_custom(f, file)
            }
        }
    }
}

fn parse_block(file: &mut File) -> Result<SBlock, ParseError> {
    parse_block_advanced(file, None, true, false, false)
}
fn parse_block_advanced(
    file: &mut File,
    assume_single_statement: Option<bool>,
    treat_closed_block_bracket_as_closing_delimeter: bool,
    treat_eof_as_closing_delimeter: bool,
    treat_closed_normal_bracket_as_closing_delimeter: bool,
) -> Result<SBlock, ParseError> {
    file.skip_whitespaces();
    // if true, parse exactly one statement. unless assume_single_statement.is_some(), this depends on whether the code block starts with { or not.
    let single_statement = if let Some(v) = assume_single_statement {
        v
    } else {
        if let Some('{') = file.get_char(file.get_pos().current_char_index) {
            file.next();
            false
        } else {
            true
        }
    };
    let mut statements = vec![];
    // each iteration of this loop parses one statement
    loop {
        file.skip_whitespaces();
        let err_start_of_this_statement = *file.get_pos();
        match file.get_char(file.get_pos().current_char_index) {
            Some(')') if treat_closed_normal_bracket_as_closing_delimeter => {
                if single_statement {
                    return Err(ParseError {
                        err:
                            ParseErrors::FoundClosingRoundBracketInSingleStatementBlockBeforeAnyStatement,
                        location: err_start_of_this_statement,
                        location_end: Some(*file.get_pos()),
                        context: vec![],
                        info: None,
                    });
                } else {
                    file.next();
                    break;
                }
            }
            Some('}') if treat_closed_block_bracket_as_closing_delimeter => {
                if single_statement {
                    return Err(ParseError {
                        err: ParseErrors::FoundClosingCurlyBracketInSingleStatementBlockBeforeAnyStatement,
                        location: err_start_of_this_statement,
                        location_end: Some(*file.get_pos()),
                        context: vec![],
                        info: None,
                    });
                } else {
                    file.next();
                    break;
                }
            }
            None if treat_eof_as_closing_delimeter => {
                break;
            }
            None => {
                return Err(ParseError {
                    err: ParseErrors::FoundEofInBlockBeforeStatementOrClosingCurlyBracket,
                    location: err_start_of_this_statement,
                    location_end: Some(*file.get_pos()),
                    context: vec![],
                    info: None,
                })
            }
            _ => (),
        }
        statements.push(parse_statement(file)?);
        match file.peek() {
            // Some('}') if treat_closed_block_bracket_as_closing_delimeter => break,
            Some(')') if treat_closed_normal_bracket_as_closing_delimeter => {
                file.next();
                break;
            }
            _ => (),
        }
        if single_statement && !statements.is_empty() {
            break;
        }
    }
    Ok(SBlock::new(statements))
}

fn parse_statement(file: &mut File) -> Result<SStatement, ParseError> {
    parse_statement_adv(file, false)
}
fn parse_statement_adv(
    file: &mut File,
    is_part_of_chain_already: bool,
) -> Result<SStatement, ParseError> {
    file.skip_whitespaces();
    let err_start_of_statement = *file.get_pos();
    let out = match file.peek() {
        Some('{') => Some(SStatementEnum::Block(parse_block(file)?).to()),
        Some('[') => {
            file.next();
            let mut v = vec![];
            let mut list = false;
            loop {
                file.skip_whitespaces();
                if let Some(']') = file.peek() {
                    file.next();
                    break;
                }
                if file[file.get_pos().current_char_index..].starts_with("...]") {
                    list = true;
                    file.next();
                    file.next();
                    file.next();
                    file.next();
                    break;
                }
                v.push(parse_statement(file)?);
            }
            Some(if list {
                SStatementEnum::List(v).to()
            } else {
                SStatementEnum::Tuple(v).to()
            })
        }
        Some('"') => {
            file.next();
            let mut buf = String::new();
            loop {
                match file.next() {
                    Some('\\') => {
                        if let Some(ch) = file.next() {
                            buf.push(match ch {
                                '\\' => '\\',
                                'n' => '\n',
                                't' => '\t',
                                '"' => '"',
                                ch => {
                                    eprintln!("Warn: Weird char escape \"\\{ch}\", will be replaced with \"{ch}\".");
                                    ch
                                },
                            })
                        }
                    }
                    Some('"') => break,
                    Some(ch) => buf.push(ch),
                    None => {
                        return Err(ParseError {
                            err: ParseErrors::FoundEofInString,
                            location: err_start_of_statement,
                            location_end: Some(*file.get_pos()),
                            context: vec![],
                            info: None,
                        })
                    }
                }
            }
            Some(SStatementEnum::Value(VDataEnum::String(buf).to()).to())
        }
        _ => None,
    };
    let mut out = if let Some(out) = out {
        out
    } else {
        let mut start = String::new();
        loop {
            fn is_delimeter(ch: char) -> bool {
                matches!(ch, '}' | ']' | ')' | '.')
            }
            let nchar = match file.peek() {
                Some(ch) if is_delimeter(ch) => Some(ch),
                _ => file.next(),
            };
            match nchar {
                Some(':') => {
                    if let Some(':') = file.peek() {
                        _ = file.next();
                        let file_pos_before_pot_type = *file.get_pos();
                        let parsed_type = parse_type(file);
                        file.skip_whitespaces();
                        if let Some('=') = file.next() {
                            let err_equals_sign = *file.get_pos();
                            let start = start.trim();
                            let derefs = start.chars().take_while(|c| *c == '*').count();
                            match parse_statement(file) {
                                Ok(v) => break v
                                    .output_to(start[derefs..].to_owned(), derefs)
                                    .force_output_type(Some(match parsed_type {
                                        Ok(v) => v,
                                        Err(mut e) => {
                                            e.context.push((
                                                format!("interpreted this as an assignment to a variable with the format <var>::<var_type> = <statement>"), Some((err_start_of_statement, Some(err_equals_sign)))
                                            ));
                                            return Err(e);
                                        }
                                    })),
                                Err(mut e) => {
                                    e.context.push((
                                        format!(
                                            "statement was supposed to be assigned to variable {start}"
                                        ),
                                        Some((err_start_of_statement, Some(err_equals_sign))),
                                    ));
                                    return Err(e);
                                }
                            }
                        }
                        file.set_pos(file_pos_before_pot_type);
                    }
                    return Ok(SStatement::new(SStatementEnum::EnumVariant(
                        start,
                        parse_statement(file)?,
                    )));
                }
                Some(ch) if ch.is_whitespace() || is_delimeter(ch) => {
                    if start.trim().is_empty() {
                        return Err(ParseError {
                            err: ParseErrors::StatementCannotStartWith(ch),
                            location: *file.get_pos(),
                            location_end: None,
                            context: vec![],
                            info: None,
                        });
                    }
                    file.skip_whitespaces();
                    // var = statement
                    if let Some('=') = file.peek() {
                        file.next();
                        let err_equals_sign = *file.get_pos();
                        let start = start.trim();
                        let derefs = start.chars().take_while(|c| *c == '*').count();
                        match parse_statement(file) {
                            Ok(v) => break v.output_to(start[derefs..].to_owned(), derefs),
                            Err(mut e) => {
                                e.context.push((
                                    format!(
                                        "statement was supposed to be assigned to variable {start}"
                                    ),
                                    Some((err_start_of_statement, Some(err_equals_sign))),
                                ));
                                return Err(e);
                            }
                        };
                    }
                    // parse normal statement
                    let start = start.trim();
                    match start {
                        "fn" => {
                            file.skip_whitespaces();
                            let mut fn_name = String::new();
                            loop {
                                match file.next() {
                                    Some('(') => break,
                                    Some(ch) => fn_name.push(ch),
                                    None => break,
                                }
                            }
                            let func = parse_function(file, Some(err_start_of_statement))?;
                            break SStatementEnum::FunctionDefinition(
                                Some(fn_name.trim().to_string()),
                                func,
                            )
                            .to();
                        }
                        "if" => {
                            // TODO: Else
                            let condition = parse_statement(file)?;
                            let then = parse_statement(file)?;
                            let mut then_else = None;
                            file.skip_whitespaces();
                            let i = file.get_pos().current_char_index;
                            if file[i..].starts_with("else ") {
                                while let Some('e' | 'l' | 's') = file.next() {}
                                then_else = Some(parse_statement(file)?);
                            }
                            break SStatementEnum::If(condition, then, then_else).to();
                        }
                        "for" => {
                            break SStatementEnum::For(
                                {
                                    file.skip_whitespaces();
                                    let mut buf = String::new();
                                    loop {
                                        if let Some(ch) = file.next() {
                                            if ch.is_whitespace() {
                                                break;
                                            }
                                            buf.push(ch);
                                        } else {
                                            break;
                                        }
                                    }
                                    buf
                                },
                                parse_statement(file)?,
                                parse_statement(file)?,
                            )
                            .to()
                        }
                        "while" => {
                            eprintln!("Warn: 'while' is now 'loop'. At some point, this will just be an error instead of a warning.");
                            break SStatementEnum::Loop(parse_statement(file)?).to();
                        }
                        "loop" => {
                            break SStatementEnum::Loop(parse_statement(file)?).to();
                        }
                        "switch" | "switch!" => {
                            let force = start.ends_with("!");
                            let mut switch_on_what = String::new();
                            loop {
                                match file.next() {
                                    None => break,
                                    Some(ch) if ch.is_whitespace() => break,
                                    Some(ch) => switch_on_what.push(ch),
                                }
                            }
                            file.skip_whitespaces();
                            if let Some('{') = file.next() {
                            } else {
                                eprintln!("switch statements should be followed by {{ (because they must be closed by }}). This might lead to errors when parsing, although it isn't fatal.");
                            }
                            let mut cases = vec![];
                            loop {
                                file.skip_whitespaces();
                                if let Some('}') = file.peek() {
                                    file.next();
                                    break;
                                }
                                cases.push((parse_type(file)?, parse_statement(file)?));
                            }
                            break SStatementEnum::Switch(switch_on_what, cases, force).to();
                        }
                        "match" => {
                            let mut match_what = String::new();
                            loop {
                                match file.next() {
                                    None => break,
                                    Some(ch) if ch.is_whitespace() => break,
                                    Some(ch) => match_what.push(ch),
                                }
                            }
                            file.skip_whitespaces();
                            if let Some('{') = file.next() {
                            } else {
                                eprintln!("match statements should be followed by {{ (because they must be closed by }}). This might lead to errors when parsing, although it isn't fatal.");
                            }
                            let mut cases = vec![];
                            loop {
                                file.skip_whitespaces();
                                if let Some('}') = file.peek() {
                                    file.next();
                                    break;
                                }
                                cases.push((parse_statement(file)?, parse_statement(file)?));
                            }
                            break SStatementEnum::Match(match_what, cases).to();
                        }
                        "true" => break SStatementEnum::Value(VDataEnum::Bool(true).to()).to(),
                        "false" => break SStatementEnum::Value(VDataEnum::Bool(false).to()).to(),
                        _ => {
                            // int, float, var
                            break {
                                if let Ok(v) = start.parse() {
                                    if let Some('.') = nchar {
                                        let pos = *file.get_pos();
                                        file.next();
                                        let mut pot_float = String::new();
                                        for ch in &mut *file {
                                            if ch.is_whitespace() || is_delimeter(ch) {
                                                file.set_pos(*file.get_ppos());
                                                break;
                                            }
                                            pot_float.push(ch);
                                        }
                                        if let Ok(v) = format!("{start}.{pot_float}").parse() {
                                            SStatementEnum::Value(VDataEnum::Float(v).to()).to()
                                        } else {
                                            file.set_pos(pos);
                                            SStatementEnum::Value(VDataEnum::Int(v).to()).to()
                                        }
                                    } else {
                                        SStatementEnum::Value(VDataEnum::Int(v).to()).to()
                                    }
                                // } else if let Ok(v) = start.parse() {
                                //     SStatementEnum::Value(VDataEnum::Float(v).to()).to()
                                } else {
                                    if start.starts_with('&') {
                                        SStatementEnum::Variable(start[1..].to_string(), true).to()
                                    } else {
                                        SStatementEnum::Variable(start.to_string(), false).to()
                                    }
                                }
                            };
                        }
                    }
                }
                Some('(') => {
                    // parse_block_advanced: only treat ) as closing delimeter, don't use single-statement (missing {, so would be assumed otherwise)
                    let name = start.trim();
                    if name.is_empty() {
                        break SStatementEnum::FunctionDefinition(
                            None,
                            parse_function(file, Some(err_start_of_statement))?,
                        )
                        .to();
                    } else {
                        break SStatementEnum::FunctionCall(
                            name.to_string(),
                            match parse_block_advanced(file, Some(false), false, false, true) {
                                Ok(block) => block.statements,
                                Err(e) => {
                                    // NOTE: Alternatively, just add an entry to the original error's context.
                                    return Err(ParseError {
                                        err: ParseErrors::ErrorParsingFunctionArgs(Box::new(e)),
                                        location: err_start_of_statement,
                                        location_end: Some(*file.get_pos()),
                                        context: vec![],
                                        info: None,
                                    });
                                }
                            },
                        )
                        .to();
                    }
                }
                Some(ch) => start.push(ch),
                None => {
                    return Err(ParseError {
                        err: ParseErrors::FoundEofInStatement,
                        location: err_start_of_statement,
                        location_end: Some(*file.get_pos()),
                        context: vec![],
                        info: None,
                    })
                }
            }
        }
    };
    let err_end_of_original_statement = *file.get_pos();
    file.skip_whitespaces();
    if !file[file.get_pos().current_char_index..].starts_with("..") {
        // dot chain syntax only works if there is only one dot
        if let Some('.') = file.peek() {
            // consume the dot (otherwise, a.b.c syntax will break in certain cases)
            file.next();
        }
        if !is_part_of_chain_already {
            let mut chain_length = 0;
            let mut err_end_of_prev = err_end_of_original_statement;
            while let Some('.') = file.get_char(file.get_pos().current_char_index.saturating_sub(1))
            {
                let err_start_of_wrapper = *file.get_pos();
                let wrapper = parse_statement_adv(file, true)?;
                let err_end_of_wrapper = *file.get_pos();
                out = match *wrapper.statement {
                    SStatementEnum::FunctionCall(func, args) => {
                        let args = [out].into_iter().chain(args.into_iter()).collect();
                        SStatementEnum::FunctionCall(func, args).to()
                    }
                    SStatementEnum::Value(vd) => match vd.data {
                        VDataEnum::Int(i) => SStatementEnum::IndexFixed(out, i as _).to(),
                        _ => {
                            let mut context = vec![];
                            if chain_length > 0 {
                                context.push((
                                    format!(
                                        "this is the {} wrapping statement in a chain.",
                                        match chain_length {
                                            1 => format!("second"),
                                            2 => format!("third"),
                                            // NOTE: this technically breaks at 21, but I don't care.
                                            len => format!("{}th", len + 1),
                                        }
                                    ),
                                    None,
                                ));
                            }
                            context.push((
                                format!("the statement that was supposed to be wrapped"),
                                Some((err_start_of_statement, Some(err_end_of_prev))),
                            ));
                            return Err(ParseError {
                                err: ParseErrors::CannotUseFixedIndexingWithThisType(vd.out()),
                                location: err_start_of_wrapper,
                                location_end: Some(err_end_of_wrapper),
                                context,
                                info: None,
                            });
                        }
                    },
                    other => {
                        let mut context = vec![];
                        if chain_length > 0 {
                            context.push((
                                format!(
                                    "this is the {} wrapping statement in a chain.",
                                    match chain_length {
                                        1 => format!("second"),
                                        2 => format!("third"),
                                        // NOTE: this technically breaks at 21, but I don't care.
                                        len => format!("{}th", len + 1),
                                    }
                                ),
                                None,
                            ));
                        }
                        context.push((
                            format!("the statement that was supposed to be wrapped"),
                            Some((err_start_of_statement, Some(err_end_of_prev))),
                        ));
                        return Err(ParseError {
                            err: ParseErrors::CannotWrapWithThisStatement(other),
                            location: err_start_of_wrapper,
                            location_end: Some(err_end_of_wrapper),
                            context,
                            info: None,
                        });
                    }
                };
                err_end_of_prev = err_end_of_wrapper;
                chain_length += 1;
            }
        }
    }
    Ok(out)
}

/// Assumes the function name and opening bracket have already been parsed. File should continue like "name type name type ...) <statement>"
fn parse_function(
    file: &mut File,
    err_fn_start: Option<super::file::FilePosition>,
) -> Result<SFunction, ParseError> {
    file.skip_whitespaces();
    // find the arguments to the function
    let mut args = Vec::new();
    if let Some(')') = file.peek() {
        file.next();
    } else {
        loop {
            let mut arg_name = String::new();
            loop {
                let err_fn_arg_name_start = *file.get_pos();
                match file.next() {
                    Some(ch) if ch.is_whitespace() => break,
                    Some(ch) => arg_name.push(ch),
                    None => {
                        return Err(ParseError {
                            err: ParseErrors::FoundEofInFunctionArgName,
                            location: err_fn_arg_name_start,
                            location_end: Some(*file.get_pos()),
                            context: vec![if let Some(err_fn_start) = err_fn_start {
                                (
                                    format!("the function"),
                                    Some((err_fn_start, Some(*file.get_pos()))),
                                )
                            } else {
                                (format!("not a real fn definition"), None)
                            }],
                            info: None,
                        })
                    }
                }
            }
            let (t, brk) = parse_type_adv(file, true)?;
            args.push((arg_name, t));
            if brk {
                break;
            }
        }
    }
    Ok(SFunction::new(args, parse_block(file)?))
}

pub(crate) fn parse_type(file: &mut File) -> Result<VType, ParseError> {
    match parse_type_adv(file, false) {
        Ok((v, _)) => Ok(v),
        Err(e) => Err(e),
    }
}
pub(crate) fn parse_type_adv(
    file: &mut File,
    in_fn_args: bool,
) -> Result<(VType, bool), ParseError> {
    let mut types = vec![];
    let mut closed_fn_args = false;
    loop {
        let (st, closed_bracket) = parse_single_type_adv(file, in_fn_args)?;
        types.push(st);
        if closed_bracket {
            closed_fn_args = true;
            break;
        }
        file.skip_whitespaces();
        match file.peek() {
            Some('/') => {
                file.next();
            }
            Some(')') => {
                closed_fn_args = true;
                file.next();
                break;
            }
            Some(_) => break,

            None => break,
        }
    }
    Ok((VType { types }, closed_fn_args))
}
fn parse_single_type(file: &mut File) -> Result<VSingleType, ParseError> {
    match parse_single_type_adv(file, false) {
        Ok((v, _)) => Ok(v),
        Err(e) => Err(e),
    }
}
fn parse_single_type_adv(
    file: &mut File,
    in_fn_args: bool,
) -> Result<(VSingleType, bool), ParseError> {
    file.skip_whitespaces();
    let mut closed_bracket_in_fn_args = false;
    let err_start_of_single_type = *file.get_pos();
    Ok((
        match file.next() {
            Some('&') => {
                let parse_output = parse_single_type_adv(file, in_fn_args)?;
                if parse_output.1 {
                    closed_bracket_in_fn_args = true;
                }
                VSingleType::Reference(Box::new(parse_output.0))
            }
            // Tuple or Array
            Some('[') => {
                let mut types = vec![];
                let mut list = false;
                loop {
                    file.skip_whitespaces();
                    if file[file.get_pos().current_char_index..].starts_with("...]") {
                        list = true;
                        file.next();
                        file.next();
                        file.next();
                        file.next();
                        break;
                    }
                    match file.peek() {
                        Some(']') => {
                            file.next();
                            break;
                        }
                        _ => (),
                    }
                    types.push(parse_type(file)?);
                }
                if in_fn_args {
                    file.skip_whitespaces();
                    if let Some(')') = file.peek() {
                        closed_bracket_in_fn_args = true;
                        file.next();
                    }
                }
                if list {
                    VSingleType::List(types.pop().unwrap())
                } else {
                    VSingleType::Tuple(types)
                }
            }
            Some(ch) => 'parse_single_type: {
                let mut name = ch.to_string();
                loop {
                    match file.peek() {
                        Some(']') => break,
                        Some('/') => break,
                        Some(')') if in_fn_args => {
                            file.next();
                            closed_bracket_in_fn_args = true;
                            break;
                        }
                        Some(ch) if ch.is_whitespace() => break,
                        _ => (),
                    }
                    match file.next() {
                        Some('(') => {
                            break 'parse_single_type if name.as_str() == "fn" {
                                let mut fn_types = vec![];
                                loop {
                                    file.skip_whitespaces();
                                    match file.next() {
                                        Some('(') => {
                                            let mut args = vec![];
                                            loop {
                                                let (t, fn_args_closed) =
                                                    parse_type_adv(file, true)?;
                                                args.push(t);
                                                if fn_args_closed {
                                                    break;
                                                }
                                            }
                                            let out = if let Some(v) = args.pop() {
                                                v
                                            } else {
                                                VSingleType::Tuple(vec![]).to()
                                            };
                                            fn get_all_single_types(
                                                types: &mut Vec<VType>,
                                            ) -> Vec<Vec<VSingleType>>
                                            {
                                                if types.is_empty() {
                                                    vec![]
                                                } else if types.len() == 1 {
                                                    vec![types[0].types.clone()]
                                                } else {
                                                    let last = types.pop().unwrap();
                                                    let o = get_all_single_types(types);
                                                    let mut out = Vec::with_capacity(
                                                        o.len() * last.types.len(),
                                                    );
                                                    for other in o {
                                                        for t in &last.types {
                                                            let mut vec = other.clone();
                                                            vec.push(t.clone());
                                                            out.push(vec);
                                                        }
                                                    }
                                                    types.push(last);
                                                    out
                                                }
                                            }
                                            for t in get_all_single_types(&mut args) {
                                                fn_types.push((t, out.clone()));
                                            }
                                        }
                                        Some(')') => break,
                                        Some(other) => {
                                            eprintln!("Found char '{other}' in fn type when ')' or '(' was expected (will be treated as ')'). format is fn((input11 input12 output1) (input21 input22 output2))");
                                            break;
                                        }
                                        None => {
                                            return Err(ParseError {
                                                err: ParseErrors::FoundEofInType,
                                                location: err_start_of_single_type,
                                                location_end: Some(*file.get_pos()),
                                                context: vec![],
                                                info: None,
                                            })
                                        }
                                    }
                                }
                                if in_fn_args {
                                    if let Some(')') = file.peek() {
                                        _ = file.next();
                                        closed_bracket_in_fn_args = true;
                                    }
                                }
                                VSingleType::Function(fn_types)
                            } else if name.as_str() == "thread" {
                                let inner = parse_type_adv(file, true)?;
                                if !inner.1 {
                                    eprintln!("Warn: Parsed type thread(inner_type), but might have missed the closing bracket!");
                                }
                                VSingleType::Thread(inner.0)
                            } else {
                                VSingleType::EnumVariantS(name, {
                                    let po = parse_type_adv(file, true)?;
                                    if !po.1 {
                                        // eprintln!("enum type should be closed by ')', but apparently wasn't?");
                                        assert_eq!(file.next(), Some(')'));
                                    }
                                    po.0
                                })
                            };
                        }
                        Some(ch) => name.push(ch),
                        None => {
                            return Err(ParseError {
                                err: ParseErrors::FoundEofInType,
                                location: err_start_of_single_type,
                                location_end: Some(*file.get_pos()),
                                context: vec![],
                                info: None,
                            });
                        }
                    }
                }
                match name.trim().to_lowercase().as_str() {
                    "bool" => VSingleType::Bool,
                    "int" => VSingleType::Int,
                    "float" => VSingleType::Float,
                    "string" => VSingleType::String,
                    _ => {
                        return Err(ParseError {
                            err: ParseErrors::InvalidType(name.trim().to_string()),
                            location: err_start_of_single_type,
                            location_end: Some(*file.get_pos()),
                            context: vec![],
                            info: None,
                        });
                    }
                }
            }
            None => {
                return Err(ParseError {
                    err: ParseErrors::FoundEofInsteadOfType,
                    location: err_start_of_single_type,
                    location_end: Some(*file.get_pos()),
                    context: vec![],
                    info: None,
                })
            }
        },
        closed_bracket_in_fn_args,
    ))
}
