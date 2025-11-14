use std::{collections::VecDeque, sync::Arc};

use crate::{
    data::{self, reference::ReferenceT, tuple::TupleT, Data, MersType, Type},
    errors::{CheckError, EColor, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct Tuple {
    pub pos_in_src: SourceRange,
    pub elems: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Tuple {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        let mut it = if let Some(init_to) = init_to {
            let mut vec = (0..self.elems.len())
                .map(|_| Type::empty())
                .collect::<VecDeque<_>>();
            let print_is_part_of = init_to.types.len() > 1;
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
                if let Some(t) = t.as_any().downcast_ref::<TupleT>() {
                    if t.0.len() == self.elems.len() {
                        for (i, e) in t.0.iter().enumerate() {
                            if init_to_ref {
                                vec[i].add(Arc::new(ReferenceT(e.clone())));
                            } else {
                                vec[i].add_all(e);
                            }
                        }
                    } else {
                        return Err(CheckError::new().msg(vec![
                            ("can't init a ".to_owned(), None),
                            ("tuple".to_owned(), Some(EColor::InitTo)),
                            (" with type ".to_owned(), None),
                            (t.simplified_as_string(info), Some(EColor::InitFrom)),
                            (
                                if print_is_part_of {
                                    ", which is part of ".to_owned()
                                } else {
                                    String::new()
                                },
                                None,
                            ),
                            if print_is_part_of {
                                (init_to.simplified_as_string(info), Some(EColor::InitFrom))
                            } else {
                                (String::new(), None)
                            },
                            (
                                format!(
                                    " - only tuples with the same length ({}) can be assigned",
                                    self.elems.len()
                                ),
                                None,
                            ),
                        ]));
                    }
                } else {
                    return Err(CheckError::new().msg(vec![
                        ("can't init a ".to_owned(), None),
                        ("tuple".to_owned(), Some(EColor::InitTo)),
                        (" with type ".to_owned(), None),
                        (t.simplified_as_string(info), Some(EColor::InitFrom)),
                        (
                            if print_is_part_of {
                                ", which is part of ".to_owned()
                            } else {
                                String::new()
                            },
                            None,
                        ),
                        if print_is_part_of {
                            (init_to.simplified_as_string(info), Some(EColor::InitFrom))
                        } else {
                            (String::new(), None)
                        },
                        (" - only tuples can be assigned to tuples".to_owned(), None),
                    ]));
                }
            }
            Some(vec)
        } else {
            None
        };
        Ok(Type::new(data::tuple::TupleT(
            self.elems
                .iter()
                .map(|v| {
                    v.check(
                        info,
                        if let Some(it) = &mut it {
                            Some(it.pop_front().unwrap())
                        } else {
                            None
                        }
                        .as_ref(),
                    )
                })
                .collect::<Result<_, _>>()?,
        )))
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        Ok(Data::new(data::tuple::Tuple::from(
            self.elems
                .iter()
                .map(|s| Ok(s.run(info)?))
                .collect::<Result<Vec<_>, CheckError>>()?,
        )))
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        self.elems.iter().map(|s| s.as_ref()).collect()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
