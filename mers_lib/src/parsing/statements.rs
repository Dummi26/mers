use std::{path::PathBuf, sync::Arc};

use super::{Source, SourceFrom, SourcePos};
use crate::{
    data::Data,
    errors::{error_colors, CheckError},
    program::{
        self,
        parsed::{as_type::AsType, MersStatement},
    },
};

pub fn parse(
    src: &mut Source,
    srca: &Arc<Source>,
) -> Result<Option<Box<dyn program::parsed::MersStatement>>, CheckError> {
    src.section_begin("statement".to_string());
    src.skip_whitespace();
    // type annotation:
    //  [type] statement // force output type to be `type`
    //  [[name] type] // define `name` as `type`
    //  [[name] := statement] // define `name` as the type of `statement` (`statement` is never executed)
    if matches!(src.peek_char(), Some('[')) {
        let pos_in_src = src.get_pos();
        src.next_char();
        return Ok(Some(if matches!(src.peek_char(), Some('[')) {
            src.next_char();
            // [[...
            let name = src.next_word();
            let name = name.trim().to_owned();
            src.skip_whitespace();
            if !matches!(src.next_char(), Some(']')) {
                return Err(
                    CheckError::new().msg(format!("Expected ']' after type name in [[type_name]]"))
                );
            }
            src.skip_whitespace();
            if src.peek_word_allow_colon() == ":=" {
                src.next_word_allow_colon();
                // [[name] := statement]
                let statement = match parse(src, srca) {
                    Ok(Some(v)) => v,
                    Ok(None) => {
                        return Err(CheckError::new()
                            .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                            .msg(format!("EOF after `[[...] := ...]` type definition")))
                    }
                    Err(e) => return Err(e),
                };
                if !matches!(src.next_char(), Some(']')) {
                    return Err(CheckError::new().msg(format!(
                        "Expected ']' after statement in [[type_name] := statement]"
                    )));
                }
                Box::new(program::parsed::custom_type::CustomType {
                    pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                    name,
                    source: Err(statement),
                })
            } else {
                // [[name] type]
                src.skip_whitespace();
                let as_type = super::types::parse_type(src, srca)?;
                src.skip_whitespace();
                if !matches!(src.next_char(), Some(']')) {
                    return Err(CheckError::new().msg(format!(
                        "Expected ']' after type definition in [[type_name] type_definition]"
                    )));
                }
                Box::new(program::parsed::custom_type::CustomType {
                    pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                    name,
                    source: Ok(as_type),
                })
            }
        } else {
            // [type] statement
            src.skip_whitespace();
            let type_pos_in_src = src.get_pos();
            let as_type = super::types::parse_type(src, srca)?;
            let type_pos_in_src = (type_pos_in_src, src.get_pos(), srca).into();
            src.skip_whitespace();
            if !matches!(src.next_char(), Some(']')) {
                return Err(CheckError::new()
                    .src(vec![(
                        (pos_in_src, src.get_pos(), srca).into(),
                        Some(error_colors::TypeAnnotationNoClosingBracket),
                    )])
                    .msg(format!("Missing closing bracket ']' after type annotation")));
            }
            let statement = match parse(src, srca) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                        .msg(format!("EOF after `[...]` type annotation")))
                }
                Err(e) => return Err(e),
            };
            Box::new(AsType {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                statement,
                as_type,
                type_pos_in_src,
                expand_type: true,
            })
        }));
    }
    let mut first = if let Some(s) = parse_no_chain(src, srca)? {
        s
    } else {
        return Ok(None);
    };
    let mut pos_after_first = src.get_pos();
    src.skip_whitespace();
    match src.peek_word_allow_colon() {
        ":=" => {
            let pos_in_src = src.get_pos();
            src.next_word_allow_colon();
            let source = parse(src, srca)?.ok_or_else(|| {
                CheckError::new()
                    .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                    .msg(format!("EOF after `:=`"))
            })?;
            first = Box::new(program::parsed::init_to::InitTo {
                pos_in_src: (first.source_range().start(), src.get_pos(), srca).into(),
                target: first,
                source,
            });
        }
        "=" => {
            let pos_in_src = src.get_pos();
            src.next_word_allow_colon();
            let source = parse(src, srca)?.ok_or_else(|| {
                CheckError::new()
                    .src(vec![(
                        (first.source_range().start(), src.get_pos(), srca).into(),
                        None,
                    )])
                    .msg(format!("EOF after `=`"))
            })?;
            first = Box::new(program::parsed::assign_to::AssignTo {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                target: first,
                source,
            });
        }
        "->" => {
            let pos_in_src = src.get_pos();
            src.next_word_allow_colon();
            let run = match parse(src, srca) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                        .msg(format!("EOF after `->`")))
                }
                Err(e) => return Err(e),
            };
            first = Box::new(program::parsed::function::Function {
                pos_in_src: (first.source_range().start(), src.get_pos(), srca).into(),
                arg: first,
                run,
            });
        }
        _ => loop {
            src.skip_whitespace();
            let dot_in_src = src.get_pos();
            if let Some('.') = src.peek_char() {
                src.next_char();
                let chained = match parse_no_chain(src, srca) {
                    Ok(Some(v)) => v,
                    Ok(None) => {
                        return Err(CheckError::new()
                            .src(vec![((dot_in_src, src.get_pos(), srca).into(), None)])
                            .msg(format!("EOF after `.`")))
                    }
                    Err(e) => return Err(e),
                };
                // allow a.f(b, c) syntax (but not f(a, b, c))
                if let Some('(') = src.peek_char() {
                    src.next_char();
                    let elems = parse_multiple(src, srca, ")")?;
                    first = Box::new(program::parsed::tuple::Tuple {
                        pos_in_src: (first.source_range().start(), src.get_pos(), srca).into(),
                        elems: [first].into_iter().chain(elems).collect(),
                    });
                }
                first = Box::new(program::parsed::chain::Chain {
                    pos_in_src: (first.source_range().start(), src.get_pos(), srca).into(),
                    first,
                    chained,
                });
                pos_after_first = src.get_pos();
            } else {
                src.set_pos(pos_after_first);
                break;
            }
        },
    }
    if matches!(src.peek_char(), Some(',' | ';')) {
        src.next_char();
    }
    Ok(Some(first))
}
pub fn parse_multiple(
    src: &mut Source,
    srca: &Arc<Source>,
    end: &str,
) -> Result<Vec<Box<dyn MersStatement>>, CheckError> {
    src.section_begin("block".to_string());
    let mut statements = vec![];
    loop {
        src.skip_whitespace();
        if src.peek_char().is_some_and(|ch| end.contains(ch)) {
            src.next_char();
            break;
        } else if let Some(s) = parse(src, srca)? {
            statements.push(s);
        } else {
            // EOF
            break;
        }
    }
    Ok(statements)
}
pub fn parse_no_chain(
    src: &mut Source,
    srca: &Arc<Source>,
) -> Result<Option<Box<dyn program::parsed::MersStatement>>, CheckError> {
    src.skip_whitespace();
    src.section_begin("statement no chain".to_string());
    match src.peek_char() {
        Some('#') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            if src.peek_char().is_none() {
                return Err(CheckError::new()
                    .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                    .msg(format!("EOF after #")));
            }
            if src.peek_char().is_some_and(|ch| ch.is_whitespace()) {
                src.skip_whitespace();
                return Err(CheckError::new()
                    .src(vec![(
                        (pos_in_src, src.get_pos(), srca).into(),
                        Some(error_colors::WhitespaceAfterHashtag),
                    )])
                    .msg(format!("Whitespace after #")));
            }
            match src.next_word() {
                "include" => {
                    let end_in_src = src.get_pos();
                    src.skip_whitespace();
                    let string_in_src = src.get_pos();
                    if src.next_char() == Some('"') {
                        let file_path_str = parse_string(src, srca, string_in_src)?;
                        let mut file_path: PathBuf = PathBuf::from(&file_path_str);
                        if !file_path.is_absolute() {
                            if let SourceFrom::File(other_file_path) = srca.src_from() {
                                if let Some(files_dir) = other_file_path.parent() {
                                    file_path = files_dir.join(file_path);
                                }
                            }
                        }
                        match Source::new_from_file(file_path) {
                            Ok(mut inner_src) => {
                                let inner_srca = Arc::new(inner_src.clone());
                                return Ok(Some(Box::new(
                                    program::parsed::include_mers::IncludeMers {
                                        pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                                        include: match super::parse(&mut inner_src, &inner_srca) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                return Err(CheckError::new().err_with_diff_src(e))
                                            }
                                        },
                                        inner_src,
                                    },
                                )));
                            }
                            Err(e) => {
                                return Err(CheckError::new()
                                    .src(vec![
                                        ((pos_in_src, end_in_src, srca).into(), None),
                                        (
                                            (string_in_src, src.get_pos(), srca).into(),
                                            Some(error_colors::HashIncludeCantLoadFile),
                                        ),
                                    ])
                                    .msg(format!("Can't load file '{file_path_str}': {e}")));
                            }
                        }
                    } else {
                        return Err(CheckError::new()
                            .src(vec![
                                ((pos_in_src, end_in_src, srca).into(), None),
                                ((string_in_src, src.get_pos(), srca).into(), Some(error_colors::HashIncludeNotAString)),
                            ])
                            .msg(format!(
                                "#include must be followed by a string literal like \"file.mers\" (\" expected)."
                            )));
                    }
                }
                other => {
                    let msg = format!("Unknown #statement: {other}");
                    return Err(CheckError::new()
                        .src(vec![(
                            (pos_in_src, src.get_pos(), srca).into(),
                            Some(error_colors::HashUnknown),
                        )])
                        .msg(msg));
                }
            }
        }
        Some('{') => {
            // try: is this an object?
            let pos_in_src = src.get_pos();
            src.next_char();
            let pos_in_src_after_bracket = src.get_pos();
            {
                let mut elems = vec![];
                loop {
                    src.skip_whitespace();
                    if src.peek_char() == Some('}') {
                        src.next_char();
                        return Ok(Some(Box::new(program::parsed::object::Object {
                            pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                            elems,
                        })));
                    }
                    let name = src.next_word().to_owned();
                    src.skip_whitespace();
                    match src.next_char() {
                        Some(':') if src.next_char().is_some_and(|c| c.is_whitespace()) => elems
                            .push((
                                name,
                                match parse(src, srca) {
                                    Ok(Some(v)) => v,
                                    Ok(None) => {
                                        return Err(CheckError::new()
                                            .src(vec![(
                                                (pos_in_src, src.get_pos(), srca).into(),
                                                None,
                                            )])
                                            .msg(format!("EOF after `:` in object")))
                                    }
                                    Err(e) => {
                                        return Err(CheckError::new()
                                            .src(vec![(
                                                (pos_in_src, src.get_pos(), srca).into(),
                                                None,
                                            )])
                                            .msg(format!("Error in statement after `:` in object"))
                                            .err(e))
                                    }
                                },
                            )),
                        _ => {
                            // not an object (or invalid syntax)
                            src.set_pos(pos_in_src_after_bracket);
                            break;
                        }
                    }
                }
            }
            // if not an object
            let statements = parse_multiple(src, srca, "}")?;
            return Ok(Some(Box::new(program::parsed::block::Block {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                statements,
            })));
        }
        Some('(') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            let elems = parse_multiple(src, srca, ")")?;
            return Ok(Some(Box::new(program::parsed::tuple::Tuple {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                elems,
            })));
        }
        Some('"') => {
            src.section_begin("string literal".to_string());
            let pos_in_src = src.get_pos();
            src.next_char();
            let s = parse_string(src, srca, pos_in_src)?;
            return Ok(Some(Box::new(program::parsed::value::Value {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                data: Data::new(crate::data::string::String(s)),
            })));
        }
        _ => {}
    }
    let pos_in_src = src.get_pos();
    Ok(Some(match src.next_word() {
        "if" => {
            src.section_begin("if".to_string());
            src.skip_whitespace();
            let condition = match parse(src, srca) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                        .msg(format!("EOF in `if`")))
                }
                Err(e) => return Err(e),
            };
            let on_true = match parse(src, srca) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                        .msg(format!("EOF after `if <condition>`")))
                }
                Err(e) => return Err(e),
            };
            let on_false = {
                src.skip_whitespace();
                if src.peek_word() == "else" {
                    src.section_begin("else".to_string());
                    src.next_word();
                    Some(match parse(src, srca) {
                        Ok(Some(v)) => v,
                        Ok(None) => {
                            return Err(CheckError::new()
                                .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                                .msg(format!("EOF after `else`")))
                        }
                        Err(e) => return Err(e),
                    })
                } else {
                    None
                }
            };
            Box::new(program::parsed::r#if::If {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                condition,
                on_true,
                on_false,
            })
        }
        "loop" => {
            src.section_begin("loop".to_string());
            src.skip_whitespace();
            let inner = match parse(src, srca) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos(), srca).into(), None)])
                        .msg(format!("EOF after `loop`")))
                }
                Err(e) => return Err(e),
            };
            Box::new(program::parsed::r#loop::Loop {
                pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                inner,
            })
        }
        "true" => Box::new(program::parsed::value::Value {
            pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
            data: Data::new(crate::data::bool::Bool(true)),
        }),
        "false" => Box::new(program::parsed::value::Value {
            pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
            data: Data::new(crate::data::bool::Bool(false)),
        }),
        "" => return Ok(None),
        o => {
            let o = o.to_string();
            src.section_begin("literals, variables, and other non-keyword things".to_string());
            if let Ok(n) = o.parse() {
                if src.peek_char() == Some('.') {
                    let here = src.get_pos();
                    src.next_char();
                    if let Ok(num) = format!("{o}.{}", src.next_word()).parse() {
                        Box::new(program::parsed::value::Value {
                            pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                            data: Data::new(crate::data::float::Float(num)),
                        })
                    } else {
                        src.set_pos(here);
                        Box::new(program::parsed::value::Value {
                            pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                            data: Data::new(crate::data::int::Int(n)),
                        })
                    }
                } else {
                    Box::new(program::parsed::value::Value {
                        pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                        data: Data::new(crate::data::int::Int(n)),
                    })
                }
            } else {
                if let Some('&') = o.chars().next() {
                    Box::new(program::parsed::variable::Variable {
                        pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                        is_ref: true,
                        var: o[1..].to_string(),
                    })
                } else {
                    Box::new(program::parsed::variable::Variable {
                        pos_in_src: (pos_in_src, src.get_pos(), srca).into(),
                        is_ref: false,
                        var: o.to_string(),
                    })
                }
            }
        }
    }))
}

/// expects to be called *after* a " character is consumed from src
pub fn parse_string(
    src: &mut Source,
    srca: &Arc<Source>,
    double_quote: SourcePos,
) -> Result<String, CheckError> {
    parse_string_custom_end(src, srca, double_quote, '"', '"')
}
pub fn parse_string_custom_end(
    src: &mut Source,
    srca: &Arc<Source>,
    opening: SourcePos,
    opening_char: char,
    closing_char: char,
) -> Result<String, CheckError> {
    let mut s = String::new();
    loop {
        if let Some(ch) = src.next_char() {
            if ch == '\\' {
                let backslash_in_src = src.get_pos();
                s.push(match src.next_char() {
                    Some('\\') => '\\',
                    Some('r') => '\r',
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('"') => '"',
                    Some(c) if c == closing_char || c == opening_char => c,
                    Some(o) => {
                        return Err(CheckError::new()
                            .src(vec![(
                                (backslash_in_src, src.get_pos(), srca).into(),
                                Some(error_colors::BackslashEscapeUnknown),
                            )])
                            .msg(format!("unknown backslash escape '\\{o}'")));
                    }
                    None => {
                        return Err(CheckError::new()
                            .src(vec![(
                                (backslash_in_src, src.get_pos(), srca).into(),
                                Some(error_colors::BackslashEscapeEOF),
                            )])
                            .msg(format!("EOF in backslash escape")));
                    }
                });
            } else if ch == closing_char {
                break;
            } else {
                s.push(ch);
            }
        } else {
            return Err(CheckError::new()
                .src(vec![(
                    (opening, src.get_pos(), srca).into(),
                    Some(error_colors::StringEOF),
                )])
                .msg(format!(
                    "EOF in string literal{}",
                    if closing_char != '"' {
                        format!(
                            "{opening_char}...{closing_char} (end string with '{closing_char}')"
                        )
                    } else {
                        String::new()
                    }
                )));
        }
    }
    Ok(s)
}
pub fn to_string_literal(val: &str, end: char) -> String {
    val.replace("\\", "\\\\")
        .replace("\r", "\\r")
        .replace("\n", "\\n")
        .replace("\"", "\\\"")
        .replace(end, format!("\\{end}").as_str())
}
