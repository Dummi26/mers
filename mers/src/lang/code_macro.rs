use std::{fmt::Display, fs};

use crate::parsing::{
    file::File,
    parse::{self, ParseError, ScriptError},
};

use super::{code_parsed::SStatement, code_runnable::RScript, val_data::VData, val_type::VType};

// macro format is !(macro_type [...])

#[derive(Debug)]
pub enum Macro {
    /// Compiles and executes the provided mers code at compile-time and inserts the value
    StaticMers(VData),
}

pub fn parse_macro(file: &mut File) -> Result<Macro, MacroError> {
    file.skip_whitespaces();
    let macro_type = file.collect_to_whitespace();
    Ok(match macro_type.as_str() {
        "mers" => Macro::StaticMers({
            let code = parse_mers_code(file)?;
            let mut args = vec![];
            loop {
                file.skip_whitespaces();
                if let Some(')') = file.peek() {
                    file.next();
                    break;
                }
                args.push(parse_string_val(file));
            }
            let val = code.run(args);
            if val.safe_to_share() {
                val
            } else {
                return Err(MacroError::StaticValueNotSafeToShare);
            }
        }),
        _ => return Err(MacroError::UnknownMacroType(macro_type)),
    })
}

fn parse_string_val(file: &mut File) -> String {
    parse::implementation::parse_string_val(file, |ch| ch.is_whitespace() || ch == ')')
}

fn parse_mers_code(file: &mut File) -> Result<RScript, MacroError> {
    file.skip_whitespaces();
    if let Some('{') = file.peek() {
        _ = file.next();
        match parse::parse(file) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.err.into()),
        }
    } else {
        let path = parse_string_val(file);
        #[cfg(debug_assertions)]
        eprintln!("macro: mers: path: {path}");
        let path = crate::libs::path::path_from_string(path.as_str(), file.path())
            .expect("can't include mers code because no file was found at that path");
        let mut file = File::new(
            fs::read_to_string(&path)
                .expect("can't include mers code because the file could not be read"),
            path.into(),
        );
        Ok(match parse::parse(&mut file) {
            Ok(v) => v,
            Err(e) => return Err(e.err.into()),
        })
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaticMers(v) => write!(f, "mers {v}"),
        }
    }
}

pub enum MacroError {
    MersStatementArgError(Box<ScriptError>),
    UnknownMacroType(String),
    StaticValueNotSafeToShare,
}

impl Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MersStatementArgError(e) => write!(f, "error in mers statement argument: {e}"),
            Self::UnknownMacroType(t) => write!(
                f,
                "unknown macro type '{t}', try mers-include or mers-static."
            ),
            Self::StaticValueNotSafeToShare => write!(f, "static value cannot safely be shared (cannot use value returned by mers-static in your code - maybe it was a reference, an enum, ...)"),
        }
    }
}

impl From<ScriptError> for MacroError {
    fn from(value: ScriptError) -> Self {
        Self::MersStatementArgError(Box::new(value))
    }
}
impl From<ParseError> for MacroError {
    fn from(value: ParseError) -> Self {
        let value: ScriptError = value.into();
        value.into()
    }
}
