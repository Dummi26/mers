use std::sync::{Arc, RwLock};

use crate::data::{self, Data, Type};

use super::{MersStatement, SourceRange};

#[derive(Debug)]
pub struct Variable {
    pub pos_in_src: SourceRange,
    pub is_init: bool,
    pub is_ref: bool,
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
            while info.scopes[self.var.0].vars.len() <= self.var.1 {
                info.scopes[self.var.0].vars.push(Type::empty());
            }
            info.scopes[self.var.0].vars[self.var.1] = init_to
                .expect("variable's is_init was true, but check_custom's assign was None? How?")
                .clone();
        }
        let val = if self.is_ref {
            Type::new(data::reference::ReferenceT(
                info.scopes[self.var.0].vars[self.var.1].clone(),
            ))
        } else {
            info.scopes[self.var.0].vars[self.var.1].clone()
        };
        Ok(val)
    }
    fn run_custom(&self, info: &mut super::Info) -> Data {
        if self.is_init {
            let nothing = Arc::new(RwLock::new(Data::new(data::bool::Bool(false))));
            while info.scopes[self.var.0].vars.len() <= self.var.1 {
                info.scopes[self.var.0].vars.push(Arc::clone(&nothing));
            }
            info.scopes[self.var.0].vars[self.var.1] = nothing;
        }
        if self.is_ref {
            Data::new(data::reference::Reference(Arc::clone(
                &info.scopes[self.var.0].vars[self.var.1],
            )))
        } else {
            info.scopes[self.var.0].vars[self.var.1]
                .write()
                .unwrap()
                .clone()
        }
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
