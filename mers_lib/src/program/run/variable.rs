use std::sync::{Arc, RwLock};

use crate::{
    data::{self, Data, Type},
    errors::{CheckError, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct Variable {
    pub pos_in_src: SourceRange,
    pub is_init: bool,
    // if `is_init` is true, this must also be true unless using the "ignore" `_` pattern
    pub is_ref_not_ignore: bool,
    pub var: (usize, usize),
}

impl MersStatement for Variable {
    fn has_scope(&self) -> bool {
        false
    }
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        if self.is_init {
            if self.is_ref_not_ignore {
                while info.scopes[self.var.0].vars.len() <= self.var.1 {
                    info.scopes[self.var.0].vars.push(Type::empty());
                }
                info.scopes[self.var.0].vars[self.var.1] = init_to
                    .expect("variable's is_init was true, but check_custom's assign was None? How?")
                    .clone();
            } else {
                return Ok(Type::new(data::reference::ReferenceT(
                    init_to
                        .expect("var's is_init was true, but init_to was None???")
                        .clone(),
                )));
            }
        }
        let val = if self.is_ref_not_ignore {
            Type::new(data::reference::ReferenceT(
                info.scopes[self.var.0].vars[self.var.1].clone(),
            ))
        } else {
            info.scopes[self.var.0].vars[self.var.1].clone()
        };
        Ok(val)
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        if self.is_init {
            if self.is_ref_not_ignore {
                let nothing = Arc::new(RwLock::new(Data::new(data::bool::Bool(false))));
                while info.scopes[self.var.0].vars.len() <= self.var.1 {
                    info.scopes[self.var.0].vars.push(Arc::clone(&nothing));
                }
                info.scopes[self.var.0].vars[self.var.1] = nothing;
            } else {
                // (reference to) data which will never be referenced again
                return Ok(Data::new(data::reference::Reference(Arc::new(
                    RwLock::new(Data::empty_tuple()),
                ))));
            }
        }
        Ok(if self.is_ref_not_ignore {
            let v = &info.scopes[self.var.0].vars[self.var.1];
            Data::new(data::reference::Reference(Arc::clone(v)))
        } else {
            info.scopes[self.var.0].vars[self.var.1]
                .write()
                .unwrap()
                .clone()
        })
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
