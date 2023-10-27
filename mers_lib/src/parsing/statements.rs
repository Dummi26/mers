use super::Source;
use crate::{
    data::Data,
    program::{self, parsed::MersStatement},
};

pub fn parse(src: &mut Source) -> Result<Option<Box<dyn program::parsed::MersStatement>>, ()> {
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
            let source = parse(src)?.expect("todo");
            first = Box::new(program::parsed::init_to::InitTo {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                target: first,
                source,
            });
        }
        "=" => {
            let pos_in_src = src.get_pos();
            src.next_word();
            let source = parse(src)?.expect("todo");
            first = Box::new(program::parsed::assign_to::AssignTo {
                pos_in_src: (pos_in_src, src.get_pos()).into(),
                target: first,
                source,
            });
        }
        "->" => {
            let pos_in_src = src.get_pos();
            src.next_word();
            let run = parse(src)?.expect("err: bad eof, fn needs some statement");
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
                let chained = parse_no_chain(src)?.expect("err: EOF instead of chain");
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
pub fn parse_multiple(src: &mut Source, end: &str) -> Result<Vec<Box<dyn MersStatement>>, ()> {
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
) -> Result<Option<Box<dyn program::parsed::MersStatement>>, ()> {
    src.section_begin("statement no chain".to_string());
    src.skip_whitespace();
    match src.peek_char() {
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
            let mut s = String::new();
            loop {
                if let Some(ch) = src.next_char() {
                    if ch == '\\' {
                        s.push(match src.next_char() {
                            Some('\\') => '\\',
                            Some('r') => '\r',
                            Some('n') => '\n',
                            Some('t') => '\t',
                            Some('"') => '"',
                            Some(o) => todo!("err: unknown backslash escape '\\{o}'"),
                            None => todo!("err: eof in backslash escape"),
                        });
                    } else if ch == '"' {
                        break;
                    } else {
                        s.push(ch);
                    }
                } else {
                    todo!("err: eof in string")
                }
            }
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
            let condition = parse(src)?.expect("err: EOF instead of condition");
            let on_true = parse(src)?.expect("err: EOF instead of on_true");
            let on_false = {
                src.skip_whitespace();
                if src.peek_word() == "else" {
                    src.section_begin("else".to_string());
                    src.next_word();
                    Some(parse(src)?.expect("err: EOF instead of on_false after else"))
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
