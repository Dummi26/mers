use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{
    data::{self, Data, Type},
    info,
    parsing::SourcePos,
};

#[cfg(feature = "run")]
pub mod assign_to;
#[cfg(feature = "run")]
pub mod block;
#[cfg(feature = "run")]
pub mod chain;
#[cfg(feature = "run")]
pub mod function;
#[cfg(feature = "run")]
pub mod r#if;
#[cfg(feature = "run")]
pub mod tuple;
#[cfg(feature = "run")]
pub mod value;
#[cfg(feature = "run")]
pub mod variable;

pub trait MersStatement: Debug + Send + Sync {
    fn check_custom(
        &self,
        info: &mut CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError>;
    fn run_custom(&self, info: &mut Info) -> Data;
    /// if true, local variables etc. will be contained inside their own scope.
    fn has_scope(&self) -> bool;
    fn pos_in_src(&self) -> &SourcePos;
    fn check(&self, info: &mut CheckInfo, assign: Option<&Type>) -> Result<Type, CheckError> {
        if self.has_scope() {
            info.create_scope();
        }
        let o = self.check_custom(info, assign);
        if self.has_scope() {
            info.end_scope();
        }
        o
    }
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

#[derive(Clone, Debug)]
pub struct CheckError(pub String);
impl Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Info = info::Info<Local>;
pub type CheckInfo = info::Info<CheckLocal>;

#[derive(Default, Clone, Debug)]
pub struct Local {
    vars: Vec<Arc<Mutex<Data>>>,
}
#[derive(Default, Clone, Debug)]
pub struct CheckLocal {
    vars: Vec<Type>,
}
impl info::Local for Local {
    type VariableIdentifier = usize;
    type VariableData = Arc<Mutex<Data>>;
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        let nothing = Arc::new(Mutex::new(Data::new(data::bool::Bool(false))));
        while self.vars.len() <= id {
            self.vars.push(Arc::clone(&nothing));
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
impl info::Local for CheckLocal {
    type VariableIdentifier = usize;
    type VariableData = Type;
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        while self.vars.len() <= id {
            self.vars.push(Type::empty());
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
