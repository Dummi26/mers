use std::{collections::HashMap, fmt::Debug};

use crate::info;

use super::run::{CheckError, SourceRange};

#[cfg(feature = "parse")]
pub mod assign_to;
#[cfg(feature = "parse")]
pub mod block;
#[cfg(feature = "parse")]
pub mod chain;
#[cfg(feature = "parse")]
pub mod function;
#[cfg(feature = "parse")]
pub mod r#if;
#[cfg(feature = "parse")]
pub mod include_mers;
#[cfg(feature = "parse")]
pub mod init_to;
#[cfg(feature = "parse")]
pub mod tuple;
#[cfg(feature = "parse")]
pub mod value;
#[cfg(feature = "parse")]
pub mod variable;

pub trait MersStatement: Debug + Send + Sync {
    fn has_scope(&self) -> bool;
    fn compile_custom(
        &self,
        info: &mut Info,
        comp: CompInfo,
    ) -> Result<Box<dyn super::run::MersStatement>, CheckError>;
    fn compile(
        &self,
        info: &mut Info,
        comp: CompInfo,
    ) -> Result<Box<dyn super::run::MersStatement>, CheckError> {
        if self.has_scope() {
            info.create_scope();
        }
        let o = self.compile_custom(info, comp);
        if self.has_scope() {
            info.end_scope();
        }
        o
    }
    fn source_range(&self) -> SourceRange;
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
    vars_count: usize,
}
impl info::Local for Local {
    type VariableIdentifier = String;
    type VariableData = (usize, usize);
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        self.vars_count += 1;
        self.vars.insert(id, value);
    }
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData> {
        self.vars.get(id)
    }
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData> {
        self.vars.get_mut(id)
    }
    fn duplicate(&self) -> Self {
        self.clone()
    }
}
