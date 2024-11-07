use std::collections::HashMap;

use crate::{
    data::{self, object::ObjectT, Data, Type},
    errors::{CheckError, EColor, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct Object {
    pub pos_in_src: SourceRange,
    pub fields: Vec<(usize, Box<dyn MersStatement>)>,
}
impl MersStatement for Object {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        let mut init_to_is_empty_type = false;
        let mut init_fields = if let Some(init_to) = init_to {
            init_to_is_empty_type = init_to.types.is_empty();
            let print_is_part_of = init_to.types.len() > 1;
            let mut init_fields = HashMap::new();
            for t in init_to.types.iter() {
                if let Some(t) = t.as_any().downcast_ref::<ObjectT>() {
                    for (field, t) in t.iter() {
                        init_fields
                            .entry(*field)
                            .or_insert_with(Type::empty)
                            .add_all(t);
                    }
                } else {
                    return Err(CheckError::new().msg(vec![
                        ("can't init an ".to_owned(), None),
                        ("object".to_owned(), Some(EColor::InitTo)),
                        (" with type ".to_owned(), None),
                        (t.simplified_as_string(info), Some(EColor::InitFrom)),
                        if print_is_part_of {
                            (", which is part of ".to_owned(), None)
                        } else {
                            (format!(""), None)
                        },
                        if print_is_part_of {
                            (init_to.simplified_as_string(info), Some(EColor::InitFrom))
                        } else {
                            (format!(""), None)
                        },
                        (
                            " - only objects can be assigned to objects".to_owned(),
                            None,
                        ),
                    ]));
                }
            }
            Some(init_fields)
        } else {
            None
        };
        Ok(Type::new(data::object::ObjectT::new(
            self.fields
                .iter()
                .map(|(field, v)| -> Result<_, CheckError> {
                    Ok((
                        *field,
                        v.check(
                            info,
                            if let Some(f) = &mut init_fields {
                                Some(if let Some(s) = f.remove(field) {
                                    s
                                } else if init_to_is_empty_type {
                                    Type::empty()
                                } else {
                                    return Err(CheckError::new().msg(vec![
                                        ("can't init an ".to_owned(), None),
                                        ("object".to_owned(), Some(EColor::InitTo)),
                                        (" with type ".to_owned(), None),
                                        (
                                            init_to.as_ref().unwrap().simplified_as_string(info),
                                            Some(EColor::InitFrom),
                                        ),
                                        (
                                            format!(
                                                " - field {} is missing",
                                                info.display_info().get_object_field_name(*field)
                                            ),
                                            None,
                                        ),
                                    ]));
                                })
                            } else {
                                None
                            }
                            .as_ref(),
                        )?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        )))
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        Ok(Data::new(data::object::Object::new(
            self.fields
                .iter()
                .map(|(n, s)| Ok::<_, CheckError>((n.clone(), s.run(info)?)))
                .collect::<Result<_, _>>()?,
        )))
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        self.fields.iter().map(|(_, s)| s.as_ref()).collect()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
