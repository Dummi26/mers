use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct Info<L: Local> {
    pub scopes: Vec<L>,
    pub global: L::Global,
}

impl<L: Local> Info<L> {
    pub fn new(global: L::Global) -> Self {
        Self {
            scopes: vec![L::default()],
            global,
        }
    }
    /// Returns self, but completely empty (even without globals).
    /// Only use this if you assume this Info will never be used.
    pub fn neverused() -> Self {
        Self::new(L::neverused_global())
    }
}

pub trait Local: Default + Debug {
    type VariableIdentifier;
    type VariableData;
    type Global: Debug + Clone;
    fn neverused_global() -> Self::Global;
    fn init_var(&mut self, id: Self::VariableIdentifier, value: Self::VariableData);
    fn get_var(&self, id: &Self::VariableIdentifier) -> Option<&Self::VariableData>;
    fn get_var_mut(&mut self, id: &Self::VariableIdentifier) -> Option<&mut Self::VariableData>;
    fn duplicate(&self) -> Self;
    fn display_info<'a>(global: &'a Self::Global) -> DisplayInfo<'a>;
}
#[derive(Clone, Copy)]
pub struct DisplayInfo<'a> {
    pub object_fields: &'a Arc<Mutex<HashMap<String, usize>>>,
    pub object_fields_rev: &'a Arc<Mutex<Vec<String>>>,
}
pub struct GetObjectFieldNameDisplay<'a>(&'a DisplayInfo<'a>, usize);
impl<'a> Display for GetObjectFieldNameDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut object_fields_rev = self.0.object_fields_rev.lock().unwrap();
        if self.1 < object_fields_rev.len() {
            write!(f, "{}", object_fields_rev[self.1])
        } else {
            let object_fields = self.0.object_fields.lock().unwrap();
            if self.1 < object_fields.len() {
                let mut ofr = object_fields
                    .iter()
                    .map(|v| v.0.clone())
                    .collect::<Vec<_>>();
                ofr.sort_by_cached_key(|v| object_fields.get(v).unwrap());
                *object_fields_rev = ofr;
            }
            write!(
                f,
                "{}",
                object_fields_rev
                    .get(self.1)
                    .map(String::as_str)
                    .unwrap_or("<UNKNOWN-OBJECT-FIELD>")
            )
        }
    }
}
impl DisplayInfo<'_> {
    /// this is almost always a constant-time operation, indexing a `Vec` with `field: usize`.
    /// And even if it isn't, the second, third, ... time will be, so there is no need to cache the returned value.
    pub fn get_object_field_name<'a>(&'a self, field: usize) -> GetObjectFieldNameDisplay<'a> {
        GetObjectFieldNameDisplay(self, field)
    }
}

impl<L: Local> Info<L> {
    pub fn create_scope(&mut self) {
        self.scopes.push(L::default())
    }
    /// WARNING: can remove the last scope, which can cause some other functions to panic. Use ONLY after a create_scope()
    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn display_info<'a>(&'a self) -> DisplayInfo<'a> {
        L::display_info(&self.global)
    }
}

impl<L: Local> Info<L> {
    pub fn init_var(&mut self, id: L::VariableIdentifier, value: L::VariableData) {
        self.scopes.last_mut().unwrap().init_var(id, value)
    }
    pub fn get_var(&self, id: &L::VariableIdentifier) -> Option<&L::VariableData> {
        self.scopes.iter().rev().find_map(|l| l.get_var(id))
    }
    pub fn get_var_mut(&mut self, id: &L::VariableIdentifier) -> Option<&mut L::VariableData> {
        self.scopes.iter_mut().rev().find_map(|l| l.get_var_mut(id))
    }
    pub fn duplicate(&self) -> Self {
        Self {
            scopes: self.scopes.iter().map(|v| v.duplicate()).collect(),
            global: self.global.clone(),
        }
    }
}
