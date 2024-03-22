use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Info<L: Local> {
    pub scopes: Vec<L>,
    pub global: L::Global,
}

impl<L: Local> Info<L> {
    /// Returns self, but completely empty (even without globals).
    /// Only use this if you assume this Info will never be used.
    pub fn neverused() -> Self {
        Self {
            scopes: vec![],
            global: L::Global::default(),
        }
    }
}

pub trait Local: Default + Debug {
    type VariableIdentifier;
    type VariableData;
    type Global: Default + Debug + Clone;
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData);
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData>;
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData>;
    fn duplicate(&self) -> Self;
}

impl<L: Local> Info<L> {
    pub fn create_scope(&mut self) {
        self.scopes.push(L::default())
    }
    /// WARNING: can remove the last scope, which can cause some other functions to panic. Use ONLY after a create_scope()
    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }
}

impl<L: Local> Local for Info<L> {
    type VariableIdentifier = L::VariableIdentifier;
    type VariableData = L::VariableData;
    type Global = ();
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        self.scopes.last_mut().unwrap().init_var(id, value)
    }
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData> {
        self.scopes.iter().rev().find_map(|l| l.get_var(id))
    }
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData> {
        self.scopes.iter_mut().rev().find_map(|l| l.get_var_mut(id))
    }
    fn duplicate(&self) -> Self {
        Self {
            scopes: vec![self.scopes[0].duplicate()],
            global: self.global.clone(),
        }
    }
}

impl<L: Local> Default for Info<L> {
    fn default() -> Self {
        Self {
            scopes: vec![L::default()],
            global: L::Global::default(),
        }
    }
}
