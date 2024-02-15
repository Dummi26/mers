use std::sync::Arc;

use crate::{
    data::{self, Type},
    errors::{error_colors, CheckError},
};

use super::Source;

/// multiple types are represented as a `Vec<ParsedType>`.
#[derive(Clone, Debug)]
pub enum ParsedType {
    Reference(Vec<Self>),
    Tuple(Vec<Vec<Self>>),
    Object(Vec<(String, Vec<Self>)>),
    Type(String),
    Function(Vec<(Self, Self)>),
    TypeWithInfo(String, String),
}

pub fn parse_single_type(src: &mut Source, srca: &Arc<Source>) -> Result<ParsedType, CheckError> {
    src.section_begin("parse single type".to_string());
    src.skip_whitespace();
    Ok(match src.peek_char() {
        // Reference
        Some('&') => {
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
                    return Err(CheckError::new().msg(format!(
                        "No closing ] in reference type with opening [! Found {nc} instead"
                    )));
                }
                ParsedType::Reference(types)
            } else {
                ParsedType::Reference(vec![parse_single_type(src, srca)?])
            }
        }
        // Tuple
        Some('(') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            src.section_begin("parse tuple's inner types".to_string());
            let mut inner = vec![];
            src.skip_whitespace();
            if let Some(')') = src.peek_char() {
                src.next_char();
                // empty tuple, don't even start the loop
            } else {
                loop {
                    inner.push(parse_type(src, srca)?);
                    src.skip_whitespace();
                    match src.peek_char() {
                        Some(')') => {
                            src.next_char();
                            break;
                        }
                        Some(',') => {
                            src.next_char();
                        }
                        _ => {
                            let ppos = src.get_pos();
                            src.next_char();
                            return Err(CheckError::new()
                                .src(vec![
                                    ((pos_in_src, src.get_pos(), srca).into(), None),
                                    (
                                        (ppos, src.get_pos(), srca).into(),
                                        Some(error_colors::BadCharInTupleType),
                                    ),
                                ])
                                .msg(format!(
                                "Unexpected character in tuple type, expected comma ',' or ')'."
                            )));
                        }
                    }
                }
            }
            ParsedType::Tuple(inner)
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
                            .msg(format!("Expected colon ':' in object type")));
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
                    super::statements::parse_string_custom_end(src, srca, pos, '<', '>')?,
                )
            } else {
                ParsedType::Type(t)
            }
        }
        None => todo!(),
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
        as_type.add(match t {
            ParsedType::Reference(inner) => {
                let inner = type_from_parsed(inner, info)?;
                Arc::new(data::reference::ReferenceT(inner))
            }
            ParsedType::Tuple(t) => Arc::new(data::tuple::TupleT(
                t.iter()
                    .map(|v| type_from_parsed(v, info))
                    .collect::<Result<_, _>>()?,
            )),
            ParsedType::Object(o) => Arc::new(data::object::ObjectT(
                o.iter()
                    .map(|(s, v)| -> Result<_, CheckError> {
                        Ok((s.clone(), type_from_parsed(v, info)?))
                    })
                    .collect::<Result<_, _>>()?,
            )),
            ParsedType::Type(name) => match info
                .scopes
                .iter()
                .find_map(|scope| scope.types.iter().find(|v| v.0 == name).map(|(_, v)| v))
            {
                Some(Ok(t)) => Arc::clone(t),
                Some(Err(_)) => {
                    return Err(CheckError::new().msg(format!(
                        "Type: specified type without info, but type needs additional info"
                    )))
                }
                None => return Err(CheckError::new().msg(format!("Unknown type '{name}'"))),
            },
            ParsedType::TypeWithInfo(name, additional_info) => match info
                .scopes
                .iter()
                .find_map(|scope| scope.types.iter().find(|v| v.0 == name).map(|(_, v)| v))
            {
                Some(Ok(t)) => {
                    return Err(CheckError::new().msg(format!(
                        "Type: specified type with info, but type {t} doesn't need it"
                    )))
                }
                Some(Err(f)) => f(&additional_info, info)?,
                None => return Err(CheckError::new().msg(format!("Unknown type '{name}'"))),
            },
        });
    }
    Ok(as_type)
}
