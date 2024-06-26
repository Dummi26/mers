use std::sync::Arc;

use crate::{
    data::{self, Type},
    errors::{CheckError, EColor},
};

use super::Source;

/// multiple types are represented as a `Vec<ParsedType>`.
#[derive(Clone, Debug)]
pub enum ParsedType {
    Reference(Vec<Self>),
    Tuple(Vec<Vec<Self>>),
    Object(Vec<(String, Vec<Self>)>),
    Function(Vec<(Vec<Self>, Vec<Self>)>),
    Type(String),
    TypeWithInfo(String, String),
}

pub fn parse_single_type(src: &mut Source, srca: &Arc<Source>) -> Result<ParsedType, CheckError> {
    src.section_begin("parse single type".to_string());
    src.skip_whitespace();
    Ok(match src.peek_char() {
        // Reference
        Some('&') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            if let Some('[') = src.peek_char() {
                src.next_char();
                let types = parse_type(src, srca)?;
                let nc = src.next_char();
                if !matches!(nc, Some(']')) {
                    let nc = if let Some(nc) = nc {
                        format!("'{nc}'")
                    } else {
                        format!("EOF")
                    };
                    return Err(CheckError::new()
                        .src(vec![(
                            (pos_in_src, src.get_pos(), srca).into(),
                            Some(EColor::BracketedRefTypeNoClosingBracket),
                        )])
                        .msg_str(format!(
                            "No closing ] in reference type with opening [! Found {nc} instead"
                        )));
                }
                ParsedType::Reference(types)
            } else {
                ParsedType::Reference(vec![parse_single_type(src, srca)?])
            }
        }
        // Tuple or Function
        Some('(') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            src.section_begin("parse tuple's inner types".to_string());
            let mut inner_t = vec![];
            let mut inner_f = vec![];
            src.skip_whitespace();
            if let Some(')') = src.peek_char() {
                src.next_char();
                // empty tuple, don't even start the loop
            } else {
                loop {
                    let t = parse_type(src, srca)?;
                    src.skip_whitespace();
                    match src.peek_char() {
                        Some(',' | ')') => {
                            let last = src.peek_char().is_some_and(|c| c == ')');
                            if inner_f.is_empty() {
                                inner_t.push(t);
                                src.next_char();
                                if last {
                                    break;
                                }
                            } else {
                                let pos1 = src.get_pos();
                                src.next_char();
                                return Err(CheckError::new().src(vec![
                                    ((pos_in_src, src.get_pos(), srca).into(), None),
                                    (
                                        (pos1, src.get_pos(), srca).into(),
                                        Some(EColor::BadCharInFunctionType),
                                    ),
                                ]).msg_str(format!("Unexpected character in function type, expected arrow `->` but found `,`."))
                                .msg_str(format!("If you wanted this to be a tuple type instead, you may have used `Input -> Output` instead of `(Input -> Output)` for a function type somewhere.")));
                            }
                        }
                        Some('-') if src.peek_word() == "->" => {
                            if inner_t.is_empty() {
                                src.next_word();
                                inner_f.push((t, parse_type(src, srca)?));
                                let pos2 = src.get_pos();
                                src.skip_whitespace();
                                match src.next_char() {
                                    Some(',') => (),
                                    Some(')') => break,
                                    _ => return Err(CheckError::new().src(vec![
                                        ((pos_in_src, src.get_pos(), srca).into(), None),
                                        ((pos2, src.get_pos(), srca).into(), Some(EColor::BadCharInFunctionType)),
                                    ]).msg_str(format!("Expected comma `,` after `In -> Out` part of function type")))
                                }
                            } else {
                                let pos1 = src.get_pos();
                                src.next_word();
                                return Err(CheckError::new().src(vec![
                                    ((pos_in_src, src.get_pos(), srca).into(), None),
                                    (
                                        (pos1, src.get_pos(), srca).into(),
                                        Some(EColor::BadCharInTupleType),
                                    ),
                                ]).msg_str(format!("Unexpected character in tuple type, expected comma `,` but found arrow `->`."))
                                .msg_str(format!("If you wanted to write a function type, use `(Input -> Output)` instead of `Input -> Output`.")));
                            }
                        }
                        _ => {
                            let ppos = src.get_pos();
                            src.next_char();
                            return Err(CheckError::new()
                                .src(vec![
                                    ((pos_in_src, src.get_pos(), srca).into(), None),
                                    (
                                        (ppos, src.get_pos(), srca).into(),
                                        Some(EColor::BadCharInTupleType),
                                    ),
                                ])
                                .msg_str(format!(
                                "Unexpected character in tuple type, expected comma ',' or ')'."
                            )));
                        }
                    }
                }
            }
            if inner_f.is_empty() {
                ParsedType::Tuple(inner_t)
            } else {
                ParsedType::Function(inner_f)
            }
        }
        // Object
        Some('{') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            src.section_begin("parse tuple's inner types".to_string());
            let mut inner = vec![];
            src.skip_whitespace();
            if let Some('}') = src.peek_char() {
                src.next_char();
                // empty object, don't even start the loop
            } else {
                loop {
                    src.skip_whitespace();
                    let field = src.next_word().to_owned();
                    src.skip_whitespace();
                    if src.next_char() != Some(':') {
                        return Err(CheckError::new()
                            .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                            .msg_str(format!("Expected colon ':' in object type")));
                    }
                    src.skip_whitespace();
                    inner.push((field, parse_type(src, srca)?));
                    src.skip_whitespace();
                    match src.peek_char() {
                        Some('}') => {
                            src.next_char();
                            break;
                        }
                        Some(',') => {
                            src.next_char();
                        }
                        _ => (),
                    }
                }
            }
            ParsedType::Object(inner)
        }
        Some(_) => {
            let t = src.next_word().to_owned();
            src.skip_whitespace();
            if let Some('<') = src.peek_char() {
                let pos = src.get_pos();
                src.next_char();
                ParsedType::TypeWithInfo(
                    t,
                    super::statements::parse_string_custom_end(
                        src,
                        srca,
                        pos,
                        '<',
                        '>',
                        "type-info ",
                        EColor::TypeEOF,
                    )?,
                )
            } else {
                ParsedType::Type(t)
            }
        }
        None => {
            return Err(CheckError::new()
                .src(vec![(
                    (src.get_pos_last_char(), src.get_pos(), srca).into(),
                    Some(EColor::TypeEOF),
                )])
                .msg_str(format!("Expected type, got EOF")))
        }
    })
}

pub fn parse_type(src: &mut Source, srca: &Arc<Source>) -> Result<Vec<ParsedType>, CheckError> {
    src.section_begin("parse single type".to_string());
    let mut types = vec![];
    loop {
        types.push(parse_single_type(src, srca)?);
        src.skip_whitespace();
        if let Some('/') = src.peek_char() {
            src.next_char();
            continue;
        } else {
            break;
        }
    }
    Ok(types)
}
pub fn type_from_parsed(
    parsed: &Vec<ParsedType>,
    info: &crate::program::run::CheckInfo,
) -> Result<Type, CheckError> {
    let mut as_type = Type::empty();
    for t in parsed.iter() {
        match t {
            ParsedType::Reference(inner) => {
                let inner = type_from_parsed(inner, info)?;
                as_type.add(Arc::new(data::reference::ReferenceT(inner)));
            }
            ParsedType::Tuple(t) => as_type.add(Arc::new(data::tuple::TupleT(
                t.iter()
                    .map(|v| type_from_parsed(v, info))
                    .collect::<Result<_, _>>()?,
            ))),
            ParsedType::Object(o) => as_type.add(Arc::new(data::object::ObjectT(
                o.iter()
                    .map(|(s, v)| -> Result<_, CheckError> {
                        Ok((s.clone(), type_from_parsed(v, info)?))
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            ParsedType::Function(v) => as_type.add(Arc::new(data::function::FunctionT(Err(v
                .iter()
                .map(|(i, o)| Ok((type_from_parsed(i, info)?, type_from_parsed(o, info)?)))
                .collect::<Result<_, CheckError>>()?)))),
            ParsedType::Type(name) => match info
                .scopes
                .iter()
                .find_map(|scope| scope.types.iter().find(|v| v.0 == name).map(|(_, v)| v))
            {
                Some(Ok(t)) => as_type.add_all(&*t),
                Some(Err(_)) => {
                    return Err(CheckError::new().msg_str(format!(
                        "Type: specified type without info, but type needs additional info"
                    )))
                }
                None => return Err(CheckError::new().msg_str(format!("Unknown type '{name}'"))),
            },
            ParsedType::TypeWithInfo(name, additional_info) => match info
                .scopes
                .iter()
                .find_map(|scope| scope.types.iter().find(|v| v.0 == name).map(|(_, v)| v))
            {
                Some(Ok(t)) => {
                    return Err(CheckError::new().msg_str(format!(
                        "Type: specified type with info, but type {t} doesn't need it"
                    )))
                }
                Some(Err(f)) => as_type.add_all(&*f(&additional_info, info)?),
                None => return Err(CheckError::new().msg_str(format!("Unknown type '{name}'"))),
            },
        }
    }
    Ok(as_type)
}
