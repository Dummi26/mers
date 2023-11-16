use std::fs;

use super::{Source, SourcePos};
use crate::{
    data::Data,
    program::{
        self,
        parsed::MersStatement,
        run::{error_colors, CheckError},
    },
};

pub fn parse(
    src: &mut Source,
) -> Result<Option<Box<dyn program::parsed::MersStatement>>, CheckError> {
    src.section_begin("statement".to_string());
    let mut first = if let Some(s) = parse_no_chain(src)? {
        s
    } else {
        return Ok(None);
    };
    src.skip_whitespace();
    match src.peek_word() {
        ":=" => {
            let pos_in_src = src.get_pos();
            src.next_word();
            let source = parse(src)?.ok_or_else(|| {
                CheckError::new()
                    .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                    .msg(format!("EOF after `:=`"))
            })?;
            first = Box::new(program::parsed::init_to::InitTo {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                target: first,
                source,
            });
        }
        "=" => {
            let pos_in_src = src.get_pos();
            src.next_word();
            let source = parse(src)?.ok_or_else(|| {
                CheckError::new()
                    .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                    .msg(format!("EOF after `=`"))
            })?;
            first = Box::new(program::parsed::assign_to::AssignTo {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                target: first,
                source,
            });
        }
        "->" => {
            let pos_in_src = src.get_pos();
            src.next_word();
            let run = match parse(src) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                        .msg(format!("EOF after `->`")))
                }
                Err(e) => return Err(e),
            };
            first = Box::new(program::parsed::function::Function {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                arg: first,
                run,
            });
        }
        _ => loop {
            let pos_in_src = src.get_pos();
            src.skip_whitespace();
            if let Some('.') = src.peek_char() {
                src.next_char();
                let chained = match parse_no_chain(src) {
                    Ok(Some(v)) => v,
                    Ok(None) => {
                        return Err(CheckError::new()
                            .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                            .msg(format!("EOF after `.`")))
                    }
                    Err(e) => return Err(e),
                };
                // allow a.f(b, c) syntax (but not f(a, b, c))
                if let Some('(') = src.peek_char() {
                    src.next_char();
                    let elems = parse_multiple(src, ")")?;
                    first = Box::new(program::parsed::tuple::Tuple {
                        pos_in_src: (first.source_range().start(), src.get_pos()).into(),
                        elems: [first].into_iter().chain(elems).collect(),
                    });
                }
                first = Box::new(program::parsed::chain::Chain {
                    pos_in_src: (pos_in_src, src.get_pos()).into(),
                    first,
                    chained,
                });
            } else {
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
    end: &str,
) -> Result<Vec<Box<dyn MersStatement>>, CheckError> {
    src.section_begin("block".to_string());
    let mut statements = vec![];
    loop {
        src.skip_whitespace();
        if src.peek_char().is_some_and(|ch| end.contains(ch)) {
            src.next_char();
            break;
        } else if let Some(s) = parse(src)? {
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
) -> Result<Option<Box<dyn program::parsed::MersStatement>>, CheckError> {
    src.section_begin("statement no chain".to_string());
    src.skip_whitespace();
    match src.peek_char() {
        Some('#') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            if src.peek_char().is_none() {
                return Err(CheckError::new()
                    .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                    .msg(format!("EOF after #")));
            }
            if src.peek_char().is_some_and(|ch| ch.is_whitespace()) {
                src.skip_whitespace();
                return Err(CheckError::new()
                    .src(vec![(
                        (pos_in_src, src.get_pos()).into(),
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
                        let s = parse_string(src, string_in_src)?;
                        match fs::read_to_string(&s) {
                            Ok(s) => {
                                return Ok(Some(Box::new(
                                    program::parsed::include_mers::IncludeMers {
                                        pos_in_src: (pos_in_src, src.get_pos()).into(),
                                        include: super::parse(&mut Source::new(s))?,
                                    },
                                )));
                            }
                            Err(e) => {
                                return Err(CheckError::new()
                                    .src(vec![
                                        ((pos_in_src, end_in_src).into(), None),
                                        (
                                            (string_in_src, src.get_pos()).into(),
                                            Some(error_colors::HashIncludeCantLoadFile),
                                        ),
                                    ])
                                    .msg(format!("Can't load file '{s}': {e}")));
                            }
                        }
                    } else {
                        return Err(CheckError::new()
                            .src(vec![
                                ((pos_in_src, end_in_src).into(), None),
                                ((string_in_src, src.get_pos()).into(), Some(error_colors::HashIncludeNotAString)),
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
                            (pos_in_src, src.get_pos()).into(),
                            Some(error_colors::HashUnknown),
                        )])
                        .msg(msg));
                }
            }
        }
        Some('{') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            let statements = parse_multiple(src, "}")?;
            return Ok(Some(Box::new(program::parsed::block::Block {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                statements,
            })));
        }
        Some('(') => {
            let pos_in_src = src.get_pos();
            src.next_char();
            let elems = parse_multiple(src, ")")?;
            return Ok(Some(Box::new(program::parsed::tuple::Tuple {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                elems,
            })));
        }
        Some('"') => {
            src.section_begin("string literal".to_string());
            let pos_in_src = src.get_pos();
            src.next_char();
            let s = parse_string(src, pos_in_src)?;
            return Ok(Some(Box::new(program::parsed::value::Value {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
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
            let condition = match parse(src) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                        .msg(format!("EOF in `if`")))
                }
                Err(e) => return Err(e),
            };
            let on_true = match parse(src) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(CheckError::new()
                        .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                        .msg(format!("EOF after `if <condition>`")))
                }
                Err(e) => return Err(e),
            };
            let on_false = {
                src.skip_whitespace();
                if src.peek_word() == "else" {
                    src.section_begin("else".to_string());
                    src.next_word();
                    Some(match parse(src) {
                        Ok(Some(v)) => v,
                        Ok(None) => {
                            return Err(CheckError::new()
                                .src(vec![((pos_in_src, src.get_pos()).into(), None)])
                                .msg(format!("EOF after `else`")))
                        }
                        Err(e) => return Err(e),
                    })
                } else {
                    None
                }
            };
            Box::new(program::parsed::r#if::If {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                condition,
                on_true,
                on_false,
            })
        }
        "true" => Box::new(program::parsed::value::Value {
            pos_in_src: (pos_in_src, src.get_pos()).into(),
            data: Data::new(crate::data::bool::Bool(true)),
        }),
        "false" => Box::new(program::parsed::value::Value {
            pos_in_src: (pos_in_src, src.get_pos()).into(),
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
                            pos_in_src: (pos_in_src, src.get_pos()).into(),
                            data: Data::new(crate::data::float::Float(num)),
                        })
                    } else {
                        src.set_pos(here);
                        Box::new(program::parsed::value::Value {
                            pos_in_src: (pos_in_src, src.get_pos()).into(),
                            data: Data::new(crate::data::int::Int(n)),
                        })
                    }
                } else {
                    Box::new(program::parsed::value::Value {
                        pos_in_src: (pos_in_src, src.get_pos()).into(),
                        data: Data::new(crate::data::int::Int(n)),
                    })
                }
            } else {
                if let Some('&') = o.chars().next() {
                    Box::new(program::parsed::variable::Variable {
                        pos_in_src: (pos_in_src, src.get_pos()).into(),
                        is_ref: true,
                        var: o[1..].to_string(),
                    })
                } else {
                    Box::new(program::parsed::variable::Variable {
                        pos_in_src: (pos_in_src, src.get_pos()).into(),
                        is_ref: false,
                        var: o.to_string(),
                    })
                }
            }
        }
    }))
}

/// expects to be called *after* a " character is consumed from src
pub fn parse_string(src: &mut Source, double_quote: SourcePos) -> Result<String, CheckError> {
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
                    Some(o) => {
                        return Err(CheckError::new()
                            .src(vec![(
                                (backslash_in_src, src.get_pos()).into(),
                                Some(error_colors::BackslashEscapeUnknown),
                            )])
                            .msg(format!("unknown backslash escape '\\{o}'")));
                    }
                    None => {
                        return Err(CheckError::new()
                            .src(vec![(
                                (backslash_in_src, src.get_pos()).into(),
                                Some(error_colors::BackslashEscapeEOF),
                            )])
                            .msg(format!("EOF in backslash escape")));
                    }
                });
            } else if ch == '"' {
                break;
            } else {
                s.push(ch);
            }
        } else {
            return Err(CheckError::new()
                .src(vec![(
                    (double_quote, src.get_pos()).into(),
                    Some(error_colors::StringEOF),
                )])
                .msg(format!("EOF in string literal")));
        }
    }
    Ok(s)
}
