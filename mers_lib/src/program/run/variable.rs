use crate::data::{self, Data};

use super::MersStatement;

#[derive(Debug)]
pub struct Variable {
    pub is_ref: bool,
    pub var: (usize, usize),
}

impl MersStatement for Variable {
    fn has_scope(&self) -> bool {
        false
    }
    fn run_custom(&self, info: &mut super::Info) -> Data {
        while info.scopes[self.var.0].vars.len() <= self.var.1 {
            info.scopes[self.var.0]
                .vars
                .push(Data::new(data::bool::Bool(false)));
        }
        if self.is_ref {
            Data::new(data::reference::Reference(
                info.scopes[self.var.0].vars[self.var.1].clone(),
            ))
        } else {
            // Full-Clones!
            Data::new_boxed(info.scopes[self.var.0].vars[self.var.1].get().clone())
        }
    }
}
