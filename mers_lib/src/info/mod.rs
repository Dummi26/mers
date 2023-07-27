use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Info<L: Local> {
    pub scopes: Vec<L>,
}

impl<L: Local> Info<L> {
    /// Returns self, but completely empty (even without globals).
    /// Only use this if you assume this Info will never be used.
    pub fn neverused() -> Self {
        Self { scopes: vec![] }
    }
}

pub trait Local: Default + Debug {
    type VariableIdentifier;
    type VariableData;
    // type TypesIdentifier;
    // type TypesType;
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData);
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData>;
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData>;
    // fn add_type(&mut self, id: Self::TypesIdentifier, new_type: Self::TypesType);
    // fn get_type(&self, id: Self::TypesIdentifier) -> Option<&Self::TypesType>;
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
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData) {
        self.scopes.last_mut().unwrap().init_var(id, value)
    }
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData> {
        self.scopes.iter().find_map(|l| l.get_var(id))
    }
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData> {
        self.scopes.iter_mut().find_map(|l| l.get_var_mut(id))
    }
}

impl<L: Local> Default for Info<L> {
    fn default() -> Self {
        Self {
            scopes: vec![L::default()],
        }
    }
}
