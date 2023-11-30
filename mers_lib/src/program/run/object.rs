use std::{collections::VecDeque, sync::Arc};

use colored::Colorize;

use crate::{
    data::{self, object::ObjectT, Data, Type},
    errors::{error_colors, CheckError, SourceRange},
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
                                return Err(format!("can't init an {} with type {}{} - field mismatch: {sn} != {tn}",                                    "object".color(error_colors::InitTo),
                                                t.to_string().color(error_colors::InitFrom),
                                                if print_is_part_of {
                                                    format!(
                                                        ", which is part of {}",
                                                        init_to.to_string().color(error_colors::InitFrom)
                                                    )
                                                } else {
                                                    format!("")
                                                }
                                            ).into());
                            }
                            acc[i].add(Arc::new(t.clone()));
                        }
                    } else {
                        return Err(format!(
                            "can't init an {} with type {}{} - source has {}",
                            "object".color(error_colors::InitTo),
                            t.to_string().color(error_colors::InitFrom),
                            if print_is_part_of {
                                format!(
                                    ", which is part of {}",
                                    init_to.to_string().color(error_colors::InitFrom)
                                )
                            } else {
                                format!("")
                            },
                            if self.elems.len() > t.0.len() {
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
                                    data::object::ObjectT(t.0.iter().take(self.elems.len()).cloned().collect())
                                )
                            }
                        )
                        .into());
                    }
                } else {
                    return Err(format!(
                        "can't init an {} with type {}{} - only objects can be assigned to objects",
                        "object".color(error_colors::InitTo),
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
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        Data::new(data::object::Object(
            self.elems
                .iter()
                .map(|(n, s)| (n.clone(), s.run(info)))
                .collect(),
        ))
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
}
