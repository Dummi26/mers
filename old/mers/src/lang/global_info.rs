use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use super::{
    builtins,
    fmtgs::Color,
    val_type::{VSingleType, VType},
};

pub type GSInfo = Arc<GlobalScriptInfo>;

pub struct GlobalScriptInfo {
    pub libs: Vec<crate::libs::Lib>,

    pub main_fn_args: Vec<(String, VType)>,

    pub lib_fns: HashMap<String, (usize, usize)>,

    pub enum_variants: HashMap<String, usize>,

    pub custom_type_names: HashMap<String, usize>,
    pub custom_types: Vec<VType>,

    /// if true, trying to assign to the reference of a variable that doesn't exist yet will create and initialize that variable.
    /// if false, variables will only be initialized if this is explicitly stated.
    /// settings this to true is useful for "x = 2; x = 5;" syntax in parser implementations that don't differenciate initialization and assignment syntactically.
    pub to_runnable_automatic_initialization: bool,

    pub formatter: ColorFormatter,

    pub log: Logger,
}

pub struct ColorFormatter {
    pub mode: ColorFormatMode,
    pub bracket_colors: Vec<Color>,
    pub value_string_quotes_color: Color,
    pub value_string_content_color: Color,
    pub keyword_if_color: Color,
    pub keyword_else_color: Color,
    pub keyword_loop_color: Color,
    pub keyword_for_color: Color,
    pub keyword_switch_color: Color,
    pub keyword_match_color: Color,
    pub function_call_color: Color,
    pub function_def_fn_color: Color,
    pub function_def_name_color: Color,
    pub variable_color: Color,
}
impl Default for ColorFormatter {
    fn default() -> Self {
        Self {
            mode: ColorFormatMode::Plain,
            bracket_colors: vec![
                Color::Red,
                Color::Yellow,
                Color::Cyan,
                Color::Blue,
                Color::Magenta,
            ],
            value_string_quotes_color: Color::Grey,
            value_string_content_color: Color::Cyan,
            keyword_if_color: Color::Yellow,
            keyword_else_color: Color::Yellow,
            keyword_loop_color: Color::Yellow,
            keyword_for_color: Color::Yellow,
            keyword_switch_color: Color::Yellow,
            keyword_match_color: Color::Yellow,
            function_call_color: Color::Magenta,
            function_def_fn_color: Color::Blue,
            function_def_name_color: Color::Magenta,
            variable_color: Color::Green,
        }
    }
}
#[derive(Debug)]
pub enum ColorFormatMode {
    /// No color.
    Plain,
    /// For terminal output
    Colorize,
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
            main_fn_args: vec![],
            enum_variants: Self::default_enum_variants(),
            custom_type_names: HashMap::new(),
            custom_types: vec![],
            to_runnable_automatic_initialization: false,
            formatter: Default::default(),
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
    pub fn set_main_fn_args(&mut self, args: Vec<(String, VType)>) {
        self.main_fn_args = args;
    }
    #[allow(unused)]
    pub fn add_enum_variant(&mut self, name: String) -> usize {
        let id = self.enum_variants.len();
        self.enum_variants.insert(name, id);
        id
    }
    #[allow(unused)]
    pub fn add_custom_type(&mut self, name: String, t: VType) -> usize {
        let id = self.custom_types.len();
        self.custom_types.push(t);
        self.custom_type_names.insert(name, id);
        id
    }
}

#[derive(Debug)]
pub struct Logger {
    logs: Arc<Mutex<Vec<LogMsg>>>,

    pub after_parse: LogKind,

    pub vtype_fits_in: LogKind,
    pub vsingletype_fits_in: LogKind,
}
impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(vec![])),
            after_parse: Default::default(),
            vtype_fits_in: Default::default(),
            vsingletype_fits_in: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum LogMsg {
    AfterParse(String),
    VTypeFitsIn(VType, VType, Vec<VSingleType>),
    VSingleTypeFitsIn(VSingleType, VSingleType, bool),
}
impl Logger {
    pub fn log(&self, msg: LogMsg) {
        let kind = match msg {
            LogMsg::AfterParse(..) => &self.after_parse,
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
impl Display for LogMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AfterParse(code) => {
                write!(f, "AfterParse :: {code}")
            }
            Self::VTypeFitsIn(a, b, no) => write!(f, "VTypeFitsIn :: {a} in {b} ? -> {no:?}"),
            Self::VSingleTypeFitsIn(a, b, fits) => {
                write!(f, "VSingleTypeFitsIn :: {a} in {b} ? -> {fits}")
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LogKind {
    pub stderr: bool,
    pub log: bool,
}
impl LogKind {
    pub fn log(&self) -> bool {
        self.stderr || self.log
    }
}
