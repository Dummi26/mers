use std::collections::VecDeque;

use crate::{
    data::{self, object::ObjectT, Data, MersType, Type},
    errors::{CheckError, EColor, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct Object {
    pub pos_in_src: SourceRange,
    pub elems: Vec<(String, Box<dyn MersStatement>)>,
}
impl MersStatement for Object {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        let mut assign_types = if let Some(init_to) = init_to {
            let mut acc = (0..self.elems.len())
                .map(|_| Type::empty())
                .collect::<VecDeque<_>>();
            let print_is_part_of = init_to.types.len() > 1;
            for t in init_to.types.iter() {
                if let Some(t) = t.as_any().downcast_ref::<ObjectT>() {
                    if self.elems.len() == t.0.len() {
                        for (i, ((sn, _), (tn, t))) in self.elems.iter().zip(t.0.iter()).enumerate()
                        {
                            if sn != tn {
                                return Err(CheckError::new().msg(vec![
                                    ("can't init an ".to_owned(), None),
                                    ("object".to_owned(), Some(EColor::InitTo)),
                                    (" with type ".to_owned(), None),
                                    (t.simplified_as_string(info), Some(EColor::InitFrom)),
                                    if print_is_part_of {
                                        (", which is part of ".to_owned(), None)
                                    } else {
                                        (String::new(), None)
                                    },
                                    if print_is_part_of {
                                        (init_to.simplified_as_string(info), Some(EColor::InitFrom))
                                    } else {
                                        (String::new(), None)
                                    },
                                    (" - field mismatch: ".to_owned(), None),
                                    (sn.to_owned(), None),
                                    (" != ".to_owned(), None),
                                    (tn.to_owned(), None),
                                ]));
                            }
                            acc[i].add_all(&t);
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
                            (" - source has ".to_owned(), None),
                            (if self.elems.len() > t.0.len() {
                                format!("less fields ({}, not {})", t.0.len(), self.elems.len())
                            } else {
                                format!(
                                    "more fields. Either ignore those fields (`{}`) - or remove them from the type (`... := [{}] ...`)",
                                    t.0.iter()
                                        .skip(self.elems.len())
                                        .enumerate()
                                        .map(|(i, (n, _))| if i == 0 {
                                            format!("{n}: _")
                                        } else {
                                            format!(", {n}: _")
                                        })
                                        .collect::<String>(),
                                    data::object::ObjectT(t.0.iter().take(self.elems.len()).cloned().collect()).simplified_as_string(info)
                                )
                            }, None)
                        ]));
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
            Some(acc)
        } else {
            None
        };
        Ok(Type::new(data::object::ObjectT(
            self.elems
                .iter()
                .map(|(n, v)| -> Result<_, CheckError> {
                    Ok((
                        n.clone(),
                        v.check(
                            info,
                            if let Some(it) = &mut assign_types {
                                Some(it.pop_front().unwrap())
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
        Ok(Data::new(data::object::Object(
            self.elems
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
        self.elems.iter().map(|(_, s)| s.as_ref()).collect()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
