use std::{collections::HashMap, fmt::Debug};

use crate::info;

pub mod assign_to;
pub mod block;
pub mod chain;
pub mod function;
pub mod r#if;
pub mod init_to;
pub mod r#loop;
pub mod switch;
pub mod tuple;
pub mod value;
pub mod variable;

pub trait MersStatement: Debug {
    fn has_scope(&self) -> bool;
    fn compile_custom(
        &self,
        info: &mut Info,
        comp: CompInfo,
    ) -> Result<Box<dyn super::run::MersStatement>, String>;
    fn compile(
        &self,
        info: &mut Info,
        comp: CompInfo,
    ) -> Result<Box<dyn super::run::MersStatement>, String> {
        if self.has_scope() {
            info.create_scope();
        }
        let o = self.compile_custom(info, comp);
        if self.has_scope() {
            info.end_scope();
        }
        o
    }
}

#[derive(Clone, Copy)]
pub struct CompInfo {
    is_init: bool,
}
impl Default for CompInfo {
    fn default() -> Self {
        Self { is_init: false }
    }
}

pub type Info = info::Info<Local>;

#[derive(Default, Clone, Debug)]
pub struct Local {
    vars: HashMap<String, (usize, usize)>,
}
impl info::Local for Local {
    type VariableIdentifier = String;
    type VariableData = (usize, usize);
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        self.vars.insert(id, value);
    }
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData> {
        self.vars.get(id)
    }
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData> {
        self.vars.get_mut(id)
    }
}
