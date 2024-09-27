use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use crate::{
    errors::{CheckError, SourceRange},
    info::{self, DisplayInfo},
};

#[cfg(feature = "parse")]
pub mod as_type;
#[cfg(feature = "parse")]
pub mod assign_to;
#[cfg(feature = "parse")]
pub mod block;
#[cfg(feature = "parse")]
pub mod chain;
#[cfg(feature = "parse")]
pub mod custom_type;
#[cfg(feature = "parse")]
pub mod function;
#[cfg(feature = "parse")]
pub mod r#if;
#[cfg(feature = "parse")]
pub mod include_mers;
#[cfg(feature = "parse")]
pub mod init_to;
#[cfg(feature = "parse")]
pub mod r#loop;
#[cfg(feature = "parse")]
pub mod object;
#[cfg(feature = "parse")]
pub mod r#try;
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
        info.global.depth += 1;
        if self.has_scope() {
            info.create_scope();
        }
        let o = self.compile_custom(info, comp);
        if info.global.enable_hooks {
            // Hooks - keep in sync with run/mod.rs/compile() hooks section
            {
                // `save_info_at` hook
                let mut save_info_at = info.global.save_info_at.try_lock().unwrap();
                if !save_info_at.is_empty() {
                    let pos_start = self.source_range().start().pos();
                    let pos_end = self.source_range().end().pos();
                    let cloned_info = Arc::new(info.clone());
                    for (save_to, save_at, deepest_statement) in save_info_at.iter_mut() {
                        if info.global.depth >= *deepest_statement
                            && pos_start <= *save_at
                            && *save_at < pos_end
                        {
                            if info.global.depth > *deepest_statement {
                                *deepest_statement = info.global.depth;
                                save_to.clear();
                            }
                            save_to.push((
                                self.source_range(),
                                Arc::clone(&cloned_info),
                                o.as_ref().map(|_| ()).map_err(|e| e.clone()),
                            ));
                        }
                    }
                }
            }
        }
        if self.has_scope() {
            info.end_scope();
        }
        info.global.depth -= 1;
        o
    }
    fn source_range(&self) -> SourceRange;
    fn inner_statements(&self) -> Vec<&dyn MersStatement>;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Clone, Copy)]
pub struct CompInfo {
    pub is_init: bool,
}
impl Default for CompInfo {
    fn default() -> Self {
        Self { is_init: false }
    }
}

pub type Info = info::Info<Local>;

#[derive(Default, Clone, Debug)]
pub struct Local {
    pub vars: HashMap<String, (usize, usize)>,
    pub vars_count: usize,
}
#[derive(Clone, Debug)]
pub struct LocalGlobalInfo {
    pub depth: usize,
    pub enable_hooks: bool,
    /// ((results, byte_pos_in_src, deepest_statement))
    /// you only have to set `byte_pos_in_src`. `deepest` is used internally.
    /// These values should be initialized to `(vec![], _, 0)`, but `0` can be replaced by a minimum statement depth, i.e. `2` to exclude the outer scope (which has depth `1`).
    pub save_info_at: Arc<
        Mutex<
            Vec<(
                Vec<(SourceRange, Arc<Info>, Result<(), CheckError>)>,
                usize,
                usize,
            )>,
        >,
    >,
    pub object_fields: Arc<Mutex<HashMap<String, usize>>>,
    pub object_fields_rev: Arc<Mutex<Vec<String>>>,
}
impl LocalGlobalInfo {
    pub fn new(object_fields: Arc<Mutex<HashMap<String, usize>>>) -> Self {
        Self {
            depth: 0,
            enable_hooks: false,
            save_info_at: Default::default(),
            object_fields,
            object_fields_rev: Default::default(),
        }
    }
}
impl info::Local for Local {
    type VariableIdentifier = String;
    type VariableData = (usize, usize);
    type Global = LocalGlobalInfo;
    fn neverused_global() -> Self::Global {
        Self::Global {
            depth: 0,
            enable_hooks: false,
            save_info_at: Default::default(),
            object_fields: Default::default(),
            object_fields_rev: Default::default(),
        }
    }
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
    fn display_info<'a>(global: &'a Self::Global) -> DisplayInfo<'a> {
        DisplayInfo {
            object_fields: &global.object_fields,
            object_fields_rev: &global.object_fields_rev,
        }
    }
}
