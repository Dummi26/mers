use std::{path::PathBuf, process::Command, sync::Arc};

use crate::{
    libs,
    script::{
        block::{
            to_runnable::ToRunnableError,
            to_runnable::{self, GInfo},
            RScript, SBlock, SFunction, SStatement, SStatementEnum,
        },
        val_data::VDataEnum,
        val_type::{VSingleType, VType},
    },
};

use super::file::File;

#[derive(Debug)]
pub enum ScriptError {
    ParseError(ParseError),
    ToRunnableError(ToRunnableError),
}
impl From<ParseError> for ScriptError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}
impl From<ToRunnableError> for ScriptError {
    fn from(value: ToRunnableError) -> Self {
        Self::ToRunnableError(value)
    }
}
#[derive(Debug)]
pub enum ParseError {}

pub fn parse(file: &mut File) -> Result<RScript, ScriptError> {
    let mut libs = vec![];
    loop {
        file.skip_whitespaces();
        let pos = file.get_pos().clone();
        let line = file.next_line();
        if line.starts_with("lib ") {
            let path_to_executable: PathBuf = line[4..].into();
            let mut cmd = Command::new(&path_to_executable);
            if let Some(parent) = path_to_executable.parent() {
                cmd.current_dir(parent.clone());
            }
            match libs::Lib::launch(cmd) {
                Ok(lib) => {
                    libs.push(lib);
                    eprintln!("Loaded library!");
                }
                Err(e) => panic!(
                    "Unable to load library at {}: {e:?}",
                    path_to_executable.to_string_lossy().as_ref(),
                ),
            }
        } else {
            file.set_pos(pos);
            break;
        }
    }
    let func = SFunction::new(
        vec![(
            "args".to_string(),
            VSingleType::List(VSingleType::String.into()).into(),
        )],
        parse_block_advanced(file, Some(false), true, true, false)?,
    );
    eprintln!();
    #[cfg(debug_assertions)]
    eprintln!("Parsed: {func}");
    #[cfg(debug_assertions)]
    eprintln!("Parsed: {func:#?}");
    let run = to_runnable::to_runnable(func, GInfo::new(Arc::new(libs)))?;
    #[cfg(debug_assertions)]
    eprintln!("Runnable: {run:#?}");
    Ok(run)
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
    let mut statements = vec![];
    file.skip_whitespaces();
    let single_statement = if let Some(v) = assume_single_statement {
        v
    } else {
        if let Some('{') = file.get_char(file.get_char_index()) {
            file.next();
            false
        } else {
            true
        }
    };
    loop {
        file.skip_whitespaces();
        match file.get_char(file.get_char_index()) {
            Some(')') if treat_closed_normal_bracket_as_closing_delimeter => {
                if single_statement {
                    todo!("Err: closing function-arguments-delimeter in single-statement block before any statement (???fn with single-statement???)")
                } else {
                    file.next();
                    break;
                }
            }
            Some('}') if treat_closed_block_bracket_as_closing_delimeter => {
                if single_statement {
                    todo!("Err: closing block-delimeter in single-statement block before any statement")
                } else {
                    file.next();
                    break;
                }
            }
            None if treat_eof_as_closing_delimeter => {
                break;
            }
            None => todo!("eof in block before statement"),
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
    let mut start = String::new();
    let out = match file.peek() {
        Some('{') => Some(SStatementEnum::Block(parse_block(file)?).into()),
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
                if file[file.get_char_index()..].starts_with("...]") {
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
                SStatementEnum::List(v).into()
            } else {
                SStatementEnum::Tuple(v).into()
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
                    None => todo!("Err: EOF in string"),
                }
            }
            Some(SStatementEnum::Value(VDataEnum::String(buf).to()).into())
        }
        _ => None,
    };
    let mut out = if let Some(out) = out {
        out
    } else {
        loop {
            match match file.peek() {
                Some(ch) if matches!(ch, '}' | ']' | ')' | '.') => Some(ch),
                _ => file.next(),
            } {
                Some('=') => {
                    break parse_statement(file)?.output_to(start.trim().to_string());
                }
                Some(ch) if ch.is_whitespace() || matches!(ch, '}' | ']' | ')' | '.') => {
                    file.skip_whitespaces();
                    if let Some('=') = file.peek() {
                        continue;
                    } else {
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
                                let func = parse_function(file)?;
                                break SStatementEnum::FunctionDefinition(
                                    Some(fn_name.trim().to_string()),
                                    func,
                                )
                                .into();
                            }
                            "if" => {
                                // TODO: Else
                                let condition = parse_statement(file)?;
                                let then = parse_statement(file)?;
                                let mut then_else = None;
                                file.skip_whitespaces();
                                let i = file.get_char_index();
                                if file[i..].starts_with("else ") {
                                    while let Some('e' | 'l' | 's') = file.next() {}
                                    then_else = Some(parse_statement(file)?);
                                }
                                break SStatementEnum::If(condition, then, then_else).into();
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
                                .into()
                            }
                            "while" => {
                                break SStatementEnum::While(parse_statement(file)?).into();
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
                                break SStatementEnum::Switch(switch_on_what, cases, force).into();
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
                                break SStatementEnum::Match(match_what, cases).into();
                            }
                            "true" => {
                                break SStatementEnum::Value(VDataEnum::Bool(true).to()).into()
                            }
                            "false" => {
                                break SStatementEnum::Value(VDataEnum::Bool(false).to()).into()
                            }
                            _ => {
                                // int, float, var
                                break {
                                    if let Ok(v) = start.parse() {
                                        SStatementEnum::Value(VDataEnum::Int(v).to()).into()
                                    } else if let Ok(v) = start.replace(",", ".").parse() {
                                        SStatementEnum::Value(VDataEnum::Float(v).to()).into()
                                    } else {
                                        if start.starts_with('&') {
                                            SStatementEnum::Variable(start[1..].to_string(), true)
                                                .into()
                                        } else {
                                            SStatementEnum::Variable(start.to_string(), false)
                                                .into()
                                        }
                                    }
                                };
                            }
                        }
                    }
                }
                Some('(') => {
                    // parse_block_advanced: only treat ) as closing delimeter, don't use single-statement (missing {, so would be assumed otherwise)
                    let name = start.trim();
                    if name.is_empty() {
                        break SStatementEnum::FunctionDefinition(None, parse_function(file)?)
                            .into();
                    } else {
                        break SStatementEnum::FunctionCall(
                            name.to_string(),
                            parse_block_advanced(file, Some(false), false, false, true)?.statements,
                        )
                        .into();
                    }
                }
                Some(ch) => start.push(ch),
                None => todo!("EOF in statement"),
            }
        }
    };
    file.skip_whitespaces();
    if !file[file.get_char_index()..].starts_with("..") {
        // dot chain syntax only works if there is only one dot
        if let Some('.') = file.get_char(file.get_char_index()) {
            // consume the dot (otherwise, a.b.c syntax will break in certain cases)
            file.next();
        }
        if !is_part_of_chain_already {
            while let Some('.') = file.get_char(file.get_char_index().saturating_sub(1)) {
                let wrapper = parse_statement_adv(file, true)?;
                out = match *wrapper.statement {
                    SStatementEnum::FunctionCall(func, args) => {
                        let args = [out].into_iter().chain(args.into_iter()).collect();
                        SStatementEnum::FunctionCall(func, args).into()
                    }
                    SStatementEnum::Value(vd) => match vd.data {
                        VDataEnum::Int(i) => SStatementEnum::IndexFixed(out, i as _).into(),
                        _ => todo!("fixed-indexing not available with this type."),
                    },
                    other => {
                        todo!("Wrapping in this type isn't implemented (yet?). Type: {other:?}")
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Assumes the function name and opening bracket have already been parsed. File should continue like "name type name type ...) <statement>"
fn parse_function(file: &mut File) -> Result<SFunction, ParseError> {
    // find the arguments to the function
    let mut args = Vec::new();
    file.skip_whitespaces();
    loop {
        let mut arg_name = String::new();
        match file.next() {
            Some(')') => break,
            Some(ch) => arg_name.push(ch),
            None => break,
        }
        loop {
            match file.next() {
                Some(ch) if ch.is_whitespace() => break,
                Some(ch) => arg_name.push(ch),
                None => todo!("Err: EOF in function"),
            }
        }
        let (t, brk) = parse_type_adv(file, true)?;
        args.push((arg_name, t));
        if brk {
            break;
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
            Some(ch) => {
                break;
            }
            _ => break,
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
                loop {
                    file.skip_whitespaces();
                    match file.peek() {
                        Some(']') => {
                            file.next();
                            break;
                        }
                        _ => (),
                    }
                    types.push(parse_type(file)?);
                }
                if types.len() == 1 {
                    VSingleType::List(types.pop().unwrap())
                } else {
                    VSingleType::Tuple(types)
                }
            }
            Some(ch) => {
                let mut name = ch.to_string();
                loop {
                    match file.peek() {
                        Some(']') => break,
                        Some('/') => break,
                        _ => (),
                    }
                    match file.next() {
                        Some(ch) if ch.is_whitespace() => break,
                        Some(')') if in_fn_args => {
                            closed_bracket_in_fn_args = true;
                            break;
                        }
                        Some(ch) => name.push(ch),
                        None => todo!("Err: EOF in type"),
                    }
                }
                match name.trim().to_lowercase().as_str() {
                    "bool" => VSingleType::Bool,
                    "int" => VSingleType::Int,
                    "float" => VSingleType::Float,
                    "string" => VSingleType::String,
                    _ => todo!("Err: Invalid type: \"{}\"", name.trim()),
                }
            }
            None => todo!("Err: EOF in type (1)"),
        },
        closed_bracket_in_fn_args,
    ))
}
