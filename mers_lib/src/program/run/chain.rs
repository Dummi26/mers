use std::{any::Any, sync::Arc};

use crate::data::{function::Function, Data};

use super::MersStatement;

#[derive(Debug)]
pub struct Chain {
    pub first: Box<dyn MersStatement>,
    pub chained: Box<dyn MersStatement>,
}
impl MersStatement for Chain {
    fn run_custom(&self, info: &mut super::Info) -> Data {
        let f = self.first.run(info);
        let c = self.chained.run(info);
        let c = c.get();
        if let Some(func) = c.as_any().downcast_ref::<crate::data::function::Function>() {
            func.run(f)
        } else {
            todo!("err: not a function");
        }
    }
    fn has_scope(&self) -> bool {
        false
    }
}
