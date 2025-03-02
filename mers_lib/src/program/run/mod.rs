use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Read, Write},
    sync::{atomic::AtomicBool, Arc, Mutex, RwLock},
    time::Instant,
};

use crate::{
    data::{self, Data, Type},
    errors::{CheckError, EColor, SourceRange},
    info::{self, DisplayInfo},
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
pub mod field;
#[cfg(feature = "run")]
pub mod field_chain;
#[cfg(feature = "run")]
pub mod function;
#[cfg(feature = "run")]
pub mod r#if;
#[cfg(feature = "run")]
pub mod r#loop;
#[cfg(feature = "run")]
pub mod object;
#[cfg(feature = "run")]
pub mod r#try;
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
    fn run_custom(&self, info: &mut Info) -> Result<Data, CheckError>;
    /// if true, local variables etc. will be contained inside their own scope.
    fn has_scope(&self) -> bool;
    fn check(&self, info: &mut CheckInfo, init_to: Option<&Type>) -> Result<Type, CheckError> {
        info.global.depth += 1;
        if self.has_scope() {
            info.create_scope();
        }
        let o = self.check_custom(info, init_to);
        if info.global.enable_hooks {
            // Hooks - keep in sync with run/mod.rs/compile() hooks section
            'hook_save_info_at: {
                // `save_info_at` hook
                let mut save_info_at = if let Ok(lock) = info.global.save_info_at.try_lock() {
                    lock
                } else {
                    eprintln!(
                        "[HOOKS/save_info_at] couldn't acquire lock - result may be incomplete"
                    );
                    break 'hook_save_info_at;
                };
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
                                o.clone(),
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
    fn run(&self, info: &mut Info) -> Result<Data, CheckError> {
        if let Some(cutoff) = info.global.limit_runtime {
            if Instant::now() >= cutoff {
                return Err(CheckError::new()
                    .msg_str("maximum runtime exceeded".to_owned())
                    .src(vec![(
                        self.source_range(),
                        Some(EColor::MaximumRuntimeExceeded),
                    )]));
            }
        }
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
    fn inner_statements(&self) -> Vec<&dyn MersStatement>;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub type Info = info::Info<RunLocal>;
pub type CheckInfo = info::Info<CheckLocal>;

#[derive(Default, Clone, Debug)]
pub struct RunLocal {
    pub vars: Vec<Arc<RwLock<Data>>>,
}
#[derive(Clone)]
pub struct RunLocalGlobalInfo {
    /// if set, if `Instant::now()` is equal to or after the set `Instant`, stop the program with an error.
    pub limit_runtime: Option<Instant>,
    pub object_fields: Arc<Mutex<HashMap<String, usize>>>,
    pub object_fields_rev: Arc<Mutex<Vec<String>>>,
    pub stdin: Arc<Mutex<Option<Box<dyn Read + Send + Sync>>>>,
    pub stdout: Arc<Mutex<Option<(Box<dyn Write + Send + Sync>, Box<dyn Write + Send + Sync>)>>>,
    pub allow_process_exit_via_exit: Arc<AtomicBool>,
}
#[derive(Debug)]
#[allow(unused)]
struct RunLocalGlobalInfoDebug<'a> {
    pub limit_runtime: &'a Option<Instant>,
    pub object_fields: &'a Arc<Mutex<HashMap<String, usize>>>,
    pub object_fields_rev: &'a Arc<Mutex<Vec<String>>>,
    pub stdin: bool,
    pub stdout: bool,
    pub allow_process_exit_via_exit: bool,
}
impl Debug for RunLocalGlobalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            RunLocalGlobalInfoDebug {
                limit_runtime: &self.limit_runtime,
                object_fields: &self.object_fields,
                object_fields_rev: &self.object_fields_rev,
                stdin: self.stdin.lock().unwrap().is_some(),
                stdout: self.stdout.lock().unwrap().is_some(),
                allow_process_exit_via_exit: self
                    .allow_process_exit_via_exit
                    .load(std::sync::atomic::Ordering::Relaxed),
            }
        )
    }
}
impl RunLocalGlobalInfo {
    pub fn new(object_fields: Arc<Mutex<HashMap<String, usize>>>) -> Self {
        Self {
            limit_runtime: None,
            object_fields,
            object_fields_rev: Default::default(),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            allow_process_exit_via_exit: Arc::new(AtomicBool::new(true)),
        }
    }
}
#[derive(Default, Clone)]
pub struct CheckLocal {
    pub vars: Vec<Type>,
    pub types: HashMap<
        String,
        Result<
            Arc<Type>,
            Arc<dyn Fn(&str, &CheckInfo) -> Result<Arc<Type>, CheckError> + Send + Sync>,
        >,
    >,
}
#[derive(Clone)]
pub struct CheckLocalGlobalInfo {
    pub depth: usize,
    pub enable_hooks: bool,
    pub show_warnings: Option<Arc<dyn Fn(CheckError) + Send + Sync>>,
    /// ((results, byte_pos_in_src, deepest_statement))
    /// you only have to set `byte_pos_in_src`. `deepest` is used internally.
    /// These values should be initialized to `(vec![], _, 0)`, but `0` can be replaced by a minimum statement depth, i.e. `2` to exclude the outer scope (which has depth `1`).
    pub save_info_at: Arc<
        Mutex<
            Vec<(
                Vec<(SourceRange, Arc<CheckInfo>, Result<Type, CheckError>)>,
                usize,
                usize,
            )>,
        >,
    >,
    pub unused_try_statements: Arc<Mutex<Vec<(SourceRange, Vec<Option<SourceRange>>)>>>,
    pub object_fields: Arc<Mutex<HashMap<String, usize>>>,
    pub object_fields_rev: Arc<Mutex<Vec<String>>>,
}
impl CheckLocalGlobalInfo {
    pub fn show_warnings_to_stderr(&mut self) {
        self.show_warnings = Some(Arc::new(|e| {
            #[cfg(feature = "ecolor-term")]
            let theme = crate::errors::themes::TermDefaultTheme;
            #[cfg(not(feature = "ecolor-term"))]
            let theme = crate::errors::themes::NoTheme;
            eprintln!("{}", e.display(theme));
        }));
    }

    pub fn new(object_fields: Arc<Mutex<HashMap<String, usize>>>) -> Self {
        Self {
            depth: 0,
            enable_hooks: false,
            show_warnings: None,
            save_info_at: Default::default(),
            unused_try_statements: Default::default(),
            object_fields,
            object_fields_rev: Default::default(),
        }
    }
}
impl Debug for CheckLocalGlobalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheckLocalGlobalInfo {{ depth: {}, enable_hooks: {}, show_warnings: {}, unused_try_statements: {} }}", self.depth, self.enable_hooks, self.show_warnings.is_some(), self.unused_try_statements.lock().unwrap().len())
    }
}
impl Debug for CheckLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheckLocal {:?}, {:?}", self.vars, self.types.keys())
    }
}
impl info::Local for RunLocal {
    type VariableIdentifier = usize;
    type VariableData = Arc<RwLock<Data>>;
    type Global = RunLocalGlobalInfo;
    fn neverused_global() -> Self::Global {
        Self::Global {
            limit_runtime: None,
            object_fields: Default::default(),
            object_fields_rev: Default::default(),
            stdin: Default::default(),
            stdout: Default::default(),
            allow_process_exit_via_exit: Arc::new(AtomicBool::new(false)),
        }
    }
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
    fn display_info<'a>(global: &'a Self::Global) -> DisplayInfo<'a> {
        DisplayInfo {
            object_fields: &global.object_fields,
            object_fields_rev: &global.object_fields_rev,
        }
    }
}
impl info::Local for CheckLocal {
    type VariableIdentifier = usize;
    type VariableData = Type;
    type Global = CheckLocalGlobalInfo;
    fn neverused_global() -> Self::Global {
        Self::Global {
            depth: 0,
            enable_hooks: false,
            show_warnings: None,
            save_info_at: Default::default(),
            unused_try_statements: Default::default(),
            object_fields: Default::default(),
            object_fields_rev: Default::default(),
        }
    }
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
    fn display_info<'a>(global: &'a Self::Global) -> DisplayInfo<'a> {
        DisplayInfo {
            object_fields: &global.object_fields,
            object_fields_rev: &global.object_fields_rev,
        }
    }
}
