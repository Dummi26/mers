use std::{collections::VecDeque, sync::Arc};

use colored::Colorize;

use crate::{
    data::{self, tuple::TupleT, Data, Type},
    errors::{error_colors, SourceRange},
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
            for t in init_to.types.iter() {
                if let Some(t) = t.as_any().downcast_ref::<TupleT>() {
                    if t.0.len() == self.elems.len() {
                        for (i, e) in t.0.iter().enumerate() {
                            vec[i].add(Arc::new(e.clone()));
                        }
                    } else {
                        return Err(
                            format!("can't init a {} with type {}{} - only tuples with the same length ({}) can be assigned.",
                                "tuple".color(error_colors::InitTo),
                                t.to_string().color(error_colors::InitFrom),
                                if print_is_part_of {
                                    format!(", which is part of {}", init_to.to_string().color(error_colors::InitFrom))
                                } else {
                                    format!("")
                                },
                                self.elems.len()
                            ).into());
                    }
                } else {
                    return Err(format!(
                        "can't init a {} with type {}{} - only tuples can be assigned to tuples",
                        "tuple".color(error_colors::InitTo),
                        t.to_string().color(error_colors::InitFrom),
                        if print_is_part_of {
                            format!(
                                ", which is part of {}",
                                init_to.to_string().color(error_colors::InitFrom)
                            )
                        } else {
                            format!("")
                        }
                    )
                    .into());
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
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        Data::new(data::tuple::Tuple(
            self.elems.iter().map(|s| s.run(info)).collect(),
        ))
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
