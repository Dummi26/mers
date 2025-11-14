use std::{collections::HashMap, sync::Arc};

use crate::{
    data::{self, object::ObjectT, reference::ReferenceT, Data, Type},
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
            for (t, init_to_ref) in init_to
                .types
                .iter()
                .filter_map(|t| t.as_any().downcast_ref::<ReferenceT>())
                .flat_map(|r| r.0.types.iter().map(|t| (t, true)))
                .chain(
                    init_to
                        .types
                        .iter()
                        .filter(|t| !t.as_any().is::<ReferenceT>())
                        .map(|t| (t, false)),
                )
            {
                if let Some(ot) = t.as_any().downcast_ref::<ObjectT>() {
                    let mut fields = self.fields.iter().map(|(t, _)| *t).collect::<Vec<_>>();
                    fields.sort();
                    for (field, t) in ot.iter() {
                        if let Ok(i) = fields.binary_search(field) {
                            fields.remove(i);
                        }
                        let init_fields = init_fields.entry(*field).or_insert_with(Type::empty);
                        if init_to_ref {
                            init_fields.add(Arc::new(ReferenceT(t.clone())));
                        } else {
                            init_fields.add_all(t);
                        }
                    }
                    if !fields.is_empty() {
                        return Err(CheckError::new().msg(vec![
                            ("can't init an ".to_owned(), None),
                            ("object".to_owned(), Some(EColor::InitTo)),
                            (" from type ".to_owned(), None),
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
                                format!(" - missing fields {}", {
                                    let object_fields_rev =
                                        info.global.object_fields_rev.lock().unwrap();
                                    fields
                                        .iter()
                                        .map(|f| {
                                            object_fields_rev
                                                .get(*f)
                                                .map(|f| f.as_str())
                                                .unwrap_or("<unknown>")
                                        })
                                        .enumerate()
                                        .map(|(i, f)| {
                                            if i == 0 {
                                                f.to_owned()
                                            } else {
                                                format!(", {f}")
                                            }
                                        })
                                        .collect::<String>()
                                }),
                                None,
                            ),
                        ]));
                    }
                } else {
                    return Err(CheckError::new().msg(vec![
                        ("can't init an ".to_owned(), None),
                        ("object".to_owned(), Some(EColor::InitTo)),
                        (" from type ".to_owned(), None),
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
                                    unreachable!("type-checking earlier in check_custom() should prevent this")
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
                .collect::<Result<Vec<_>, _>>()?,
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
