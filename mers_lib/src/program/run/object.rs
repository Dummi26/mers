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
            let print_is_part_of = init_to.types.len() > 1;
            Some(
                self.elems
                    .iter()
                    .map(|(field, _)| -> Result<_, CheckError> {
                        let mut acc = Type::empty();
                        for t in init_to.types.iter() {
                            if let Some(t) = t.as_any().downcast_ref::<ObjectT>() {
                                let mut found = false;
                                for (name, assign_to) in t.0.iter() {
                                    if name == field {
                                        acc.add(Arc::new(assign_to.clone()));
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    return Err(format!(
                                        "can't init an {} with type {}{} - field {field} not found",
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
                                    ).into());
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
                                ).into());
                            }
                        }
                        Ok(acc)
                    })
                    .collect::<Result<VecDeque<Type>, CheckError>>()?,
            )
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
