use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    data::{self, Data, MersType, Type},
    errors::{CheckError, SourceRange},
    info,
};

#[cfg(feature = "run")]
pub mod as_type;
#[cfg(feature = "run")]
pub mod assign_to;
#[cfg(feature = "run")]
pub mod block;
#[cfg(feature = "run")]
pub mod chain;
#[cfg(feature = "run")]
pub mod custom_type;
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
    fn source_range(&self) -> SourceRange;
}

pub type Info = info::Info<Local>;
pub type CheckInfo = info::Info<CheckLocal>;

#[derive(Default, Clone, Debug)]
pub struct Local {
    vars: Vec<Arc<RwLock<Data>>>,
}
#[derive(Default, Clone)]
pub struct CheckLocal {
    vars: Vec<Type>,
    pub types: HashMap<
        String,
        Result<
            Arc<dyn MersType>,
            Arc<dyn Fn(&str, &CheckInfo) -> Result<Arc<dyn MersType>, CheckError> + Send + Sync>,
        >,
    >,
}
impl Debug for CheckLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheckLocal {:?}, {:?}", self.vars, self.types.keys())
    }
}
impl info::Local for Local {
    type VariableIdentifier = usize;
    type VariableData = Arc<RwLock<Data>>;
    type Global = ();
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        let nothing = Arc::new(RwLock::new(Data::new(data::bool::Bool(false))));
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
    fn duplicate(&self) -> Self {
        Self {
            vars: self
                .vars
                .iter()
                .map(|v| Arc::new(RwLock::new(v.read().unwrap().clone())))
                .collect(),
        }
    }
}
impl info::Local for CheckLocal {
    type VariableIdentifier = usize;
    type VariableData = Type;
    type Global = ();
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
    fn duplicate(&self) -> Self {
        self.clone()
    }
}
