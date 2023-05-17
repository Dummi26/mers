use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::libs;

use super::{
    builtins,
    val_data::VDataEnum,
    val_type::{VSingleType, VType},
};

pub type GSInfo = Arc<GlobalScriptInfo>;

#[derive(Debug)]
pub struct GlobalScriptInfo {
    pub libs: Vec<libs::Lib>,
    pub lib_fns: HashMap<String, (usize, usize)>,

    pub enum_variants: HashMap<String, usize>,

    pub custom_type_names: HashMap<String, usize>,
    pub custom_types: Vec<VType>,

    #[cfg(debug_assertions)]
    pub log: Logger,
}

impl GlobalScriptInfo {
    pub fn to_arc(self) -> GSInfo {
        Arc::new(self)
    }
}

impl Default for GlobalScriptInfo {
    fn default() -> Self {
        Self {
            libs: vec![],
            lib_fns: HashMap::new(),
            enum_variants: Self::default_enum_variants(),
            custom_type_names: HashMap::new(),
            custom_types: vec![],
            #[cfg(debug_assertions)]
            log: Logger::new(),
        }
    }
}
impl GlobalScriptInfo {
    pub fn default_enum_variants() -> HashMap<String, usize> {
        builtins::EVS
            .iter()
            .enumerate()
            .map(|(i, v)| (v.to_string(), i))
            .collect()
    }
}

#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct Logger {
    logs: Arc<Mutex<Vec<LogMsg>>>,

    pub vdata_clone: LogKind,
    pub vtype_fits_in: LogKind,
    pub vsingletype_fits_in: LogKind,
}
#[cfg(debug_assertions)]
impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(vec![])),
            vdata_clone: Default::default(),
            vtype_fits_in: Default::default(),
            vsingletype_fits_in: Default::default(),
        }
    }
}

#[cfg(debug_assertions)]
#[derive(Debug)]
pub enum LogMsg {
    VDataClone(Option<String>, VDataEnum, usize, usize),
    VTypeFitsIn(VType, VType, Vec<VSingleType>),
    VSingleTypeFitsIn(VSingleType, VSingleType, bool),
}
#[cfg(debug_assertions)]
impl Logger {
    pub fn log(&self, msg: LogMsg) {
        let kind = match msg {
            LogMsg::VDataClone(..) => &self.vdata_clone,
            LogMsg::VTypeFitsIn(..) => &self.vtype_fits_in,
            LogMsg::VSingleTypeFitsIn(..) => &self.vsingletype_fits_in,
        };
        if kind.stderr {
            eprintln!("{msg}");
        }
        if kind.log {
            if let Ok(mut logs) = self.logs.lock() {
                logs.push(msg);
            }
        }
    }
}
#[cfg(debug_assertions)]
impl Display for LogMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VDataClone(varname, data, src_addr, new_addr) => {
                write!(
                    f,
                    "VDataClone :: {data}  ({}{src_addr} -> {new_addr})",
                    if let Some(v) = varname {
                        format!("{v} | ")
                    } else {
                        String::new()
                    }
                )
            }
            Self::VTypeFitsIn(a, b, no) => write!(f, "VTypeFitsIn :: {a} in {b} ? -> {no:?}"),
            Self::VSingleTypeFitsIn(a, b, fits) => {
                write!(f, "VSingleTypeFitsIn :: {a} in {b} ? -> {fits}")
            }
        }
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, Default)]
pub struct LogKind {
    pub stderr: bool,
    pub log: bool,
}
#[cfg(debug_assertions)]
impl LogKind {
    pub fn log(&self) -> bool {
        self.stderr || self.log
    }
}
