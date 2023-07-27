use std::sync::Arc;

use crate::{
    data::{self, Data, Type},
    info,
};

pub mod assign_to;
pub mod block;
pub mod chain;
pub mod function;
pub mod r#if;
pub mod r#loop;
pub mod switch;
pub mod tuple;
pub mod value;
pub mod variable;

pub trait MersStatement: std::fmt::Debug {
    fn run_custom(&self, info: &mut Info) -> Data;
    /// if true, local variables etc. will be contained inside their own scope.
    fn has_scope(&self) -> bool;
    // fn outputs(&self) -> Type;
    fn run(&self, info: &mut Info) -> Data {
        if self.has_scope() {
            info.create_scope();
        }
        let o = self.run_custom(info);
        if self.has_scope() {
            info.end_scope();
        }
        o
    }
}

pub type Info = info::Info<Local>;

#[derive(Default, Clone, Debug)]
pub struct Local {
    vars: Vec<Data>,
}
impl info::Local for Local {
    type VariableIdentifier = usize;
    type VariableData = Data;
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        while self.vars.len() <= id {
            self.vars.push(Data::new(data::bool::Bool(false)));
        }
        self.vars[id] = value;
    }
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData> {
        match self.vars.get(*id) {
            Some(v) => Some(v),
            None => None,
        }
    }
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData> {
        match self.vars.get_mut(*id) {
            Some(v) => Some(v),
            None => None,
        }
    }
}
