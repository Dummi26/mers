use super::Source;

/// multiple types are represented as a `Vec<ParsedType>`.
pub enum ParsedType {
    Tuple(Vec<Vec<Self>>),
    Type(String),
}

fn parse_single_type(src: &mut Source) -> Result<ParsedType, ()> {
    src.section_begin("parse single type".to_string());
    src.skip_whitespace();
    Ok(match src.peek_char() {
        // Tuple
        Some('(') => {
            src.next_char();
            src.section_begin("parse tuple's inner types".to_string());
            let mut inner = vec![];
            loop {
                match src.peek_char() {
                    Some(')') => {
                        src.next_char();
                        break;
                    }
                    Some(',') => {
                        src.next_char();
                    }
                    _ => todo!("err: bad char in tuple inner type"),
                }
                inner.push(parse_type(src)?);
            }
            ParsedType::Tuple(inner)
        }
        Some(_) => ParsedType::Type(src.next_word().to_lowercase()),
        None => todo!(),
    })
}

fn parse_type(src: &mut Source) -> Result<Vec<ParsedType>, ()> {
    src.section_begin("parse single type".to_string());
    let mut types = vec![];
    loop {
        types.push(parse_single_type(src)?);
        src.skip_whitespace();
        if let Some('/') = src.peek_char() {
            continue;
        } else {
            break;
        }
    }
    Ok(types)
}
