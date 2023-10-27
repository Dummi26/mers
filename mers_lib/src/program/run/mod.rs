use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use colored::Colorize;
use line_span::LineSpanExt;

use crate::{
    data::{self, Data, Type},
    info,
    parsing::{Source, SourcePos},
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

#[derive(Clone, Copy, Debug)]
pub struct SourceRange {
    start: SourcePos,
    end: SourcePos,
}
impl From<(SourcePos, SourcePos)> for SourceRange {
    fn from(value: (SourcePos, SourcePos)) -> Self {
        SourceRange {
            start: value.0,
            end: value.1,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CheckError(Vec<CheckErrorComponent>);
#[derive(Clone, Debug)]
enum CheckErrorComponent {
    Message(String),
    Error(CheckError),
    Source(Vec<(SourceRange, Option<colored::Color>)>),
}
#[derive(Clone)]
pub struct CheckErrorHRConfig {
    indent_start: String,
    indent_default: String,
    indent_end: String,
}
pub struct CheckErrorDisplay<'a> {
    e: &'a CheckError,
    src: Option<&'a Source>,
}
impl Display for CheckErrorDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.e.human_readable(
            f,
            self.src,
            &CheckErrorHRConfig {
                indent_start: String::new(),
                indent_default: String::new(),
                indent_end: String::new(),
            },
        )
    }
}
impl CheckError {
    pub fn new() -> Self {
        CheckError(vec![])
    }
    fn add(mut self, v: CheckErrorComponent) -> Self {
        self.0.push(v);
        self
    }
    pub(crate) fn msg(self, s: String) -> Self {
        self.add(CheckErrorComponent::Message(s))
    }
    pub(crate) fn err(self, e: Self) -> Self {
        self.add(CheckErrorComponent::Error(e))
    }
    pub(crate) fn src(self, s: Vec<(SourceRange, Option<colored::Color>)>) -> Self {
        self.add(CheckErrorComponent::Source(s))
    }
    pub fn display<'a>(&'a self, src: &'a Source) -> CheckErrorDisplay<'a> {
        CheckErrorDisplay {
            e: self,
            src: Some(src),
        }
    }
    pub fn display_no_src<'a>(&'a self) -> CheckErrorDisplay<'a> {
        CheckErrorDisplay { e: self, src: None }
    }
    // will, unless empty, end in a newline
    fn human_readable(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        src: Option<&Source>,
        cfg: &CheckErrorHRConfig,
    ) -> std::fmt::Result {
        let len = self.0.len();
        for (i, component) in self.0.iter().enumerate() {
            macro_rules! indent {
                () => {
                    if i + 1 == len {
                        &cfg.indent_end
                    } else if i == 0 {
                        &cfg.indent_start
                    } else {
                        &cfg.indent_default
                    }
                };
            }
            match component {
                CheckErrorComponent::Message(msg) => writeln!(f, "{}{msg}", indent!())?,
                CheckErrorComponent::Error(err) => {
                    let mut cfg = cfg.clone();
                    cfg.indent_start.push_str("│");
                    cfg.indent_default.push_str("│");
                    cfg.indent_end.push_str("└");
                    err.human_readable(f, src, &cfg)?;
                }
                CheckErrorComponent::Source(highlights) => {
                    if let Some(src) = src {
                        let start = highlights.iter().map(|v| v.0.start.pos()).min();
                        let end = highlights.iter().map(|v| v.0.start.pos()).max();
                        if let (Some(start), Some(end)) = (start, end) {
                            writeln!(f, "{}Line(s) [?] ({start}..{end})", indent!())?;
                            let start = src.get_line_start(start);
                            let end = src.get_line_end(end);
                            let lines = src.src()[start..end].line_spans().collect::<Vec<_>>();
                            for line in lines {
                                let line_start = line.start();
                                let line_end = line.end();
                                let line = line.as_str();
                                writeln!(f, "{} {line}", indent!())?;
                                let mut right = 0;
                                for (pos, color) in highlights {
                                    if let Some(color) = color {
                                        let highlight_start = pos.start.pos() - start;
                                        let highlight_end = pos.end.pos() - start;
                                        if highlight_start < line_end && highlight_end > line_start
                                        {
                                            let hl_start =
                                                highlight_start.saturating_sub(line_start);
                                            if hl_start < right {
                                                right = 0;
                                                writeln!(f)?;
                                            }
                                            let hl_len = highlight_end
                                                .saturating_sub(line_start)
                                                .saturating_sub(hl_start);
                                            let hl_space = hl_start - right;
                                            let print_indent = right == 0;
                                            right += hl_space + hl_len;
                                            let hl_len =
                                                hl_len.min(highlight_end - highlight_start);
                                            if print_indent && right != 0 {
                                                write!(f, "{} ", indent!())?;
                                            }
                                            write!(
                                                f,
                                                "{}{}",
                                                " ".repeat(hl_space),
                                                "^".repeat(hl_len).color(*color)
                                            )?;
                                        }
                                    }
                                }
                                if right != 0 {
                                    writeln!(f)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
impl From<String> for CheckError {
    fn from(value: String) -> Self {
        Self::new().msg(value)
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
