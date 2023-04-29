use std::fmt::{self, Display, Formatter, Pointer};

use super::{code_macro::Macro, global_info::GlobalScriptInfo, val_data::VData, val_type::VType};

pub enum SStatementEnum {
    Value(VData),
    Tuple(Vec<SStatement>),
    List(Vec<SStatement>),
    Variable(String, bool),
    FunctionCall(String, Vec<SStatement>),
    FunctionDefinition(Option<String>, SFunction),
    Block(SBlock),
    If(SStatement, SStatement, Option<SStatement>),
    Loop(SStatement),
    For(String, SStatement, SStatement),
    Switch(String, Vec<(VType, SStatement)>, bool),
    Match(String, Vec<(SStatement, SStatement)>),
    IndexFixed(SStatement, usize),
    EnumVariant(String, SStatement),
    TypeDefinition(String, VType),
    Macro(Macro),
}
impl SStatementEnum {
    pub fn to(self) -> SStatement {
        SStatement {
            output_to: None,
            statement: Box::new(self),
            force_output_type: None,
        }
    }
}

pub struct SStatement {
    pub output_to: Option<(String, usize)>,
    pub statement: Box<SStatementEnum>,
    pub force_output_type: Option<VType>,
}

impl SStatement {
    pub fn new(statement: SStatementEnum) -> Self {
        Self {
            output_to: None,
            statement: Box::new(statement),
            force_output_type: None,
        }
    }
    pub fn output_to(mut self, var: String, derefs: usize) -> Self {
        self.output_to = Some((var, derefs));
        self
    }
    // forces the statement's output to fit in a certain type.
    pub fn force_output_type(mut self, force_output_type: Option<VType>) -> Self {
        self.force_output_type = force_output_type;
        self
    }
}

/// A block of code is a collection of statements.
pub struct SBlock {
    pub statements: Vec<SStatement>,
}
impl SBlock {
    pub fn new(statements: Vec<SStatement>) -> Self {
        Self { statements }
    }
}

// A function is a block of code that starts with some local variables as inputs and returns some value as its output. The last statement in the block will be the output.
pub struct SFunction {
    pub inputs: Vec<(String, VType)>,
    pub block: SBlock,
}
impl SFunction {
    pub fn new(inputs: Vec<(String, VType)>, block: SBlock) -> Self {
        Self { inputs, block }
    }
}

//

impl SStatementEnum {
    pub fn fmtgs(&self, f: &mut Formatter, info: Option<&GlobalScriptInfo>) -> fmt::Result {
        match self {
            Self::Value(v) => v.fmtgs(f, info),
            Self::Tuple(v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    v.fmtgs(f, info)?;
                }
                write!(f, "]")
            }
            Self::List(v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    v.fmtgs(f, info)?;
                    write!(f, " ")?;
                }
                write!(f, "...]")
            }
            Self::Variable(var, reference) => {
                if *reference {
                    write!(f, "&{var}")
                } else {
                    write!(f, "{var}")
                }
            }
            Self::FunctionCall(func, args) => {
                write!(f, "{func}(")?;
                for arg in args {
                    arg.fmtgs(f, info)?;
                }
                write!(f, ")")
            }
            Self::FunctionDefinition(name, func) => {
                if let Some(name) = name {
                    write!(f, "{name}")?;
                }
                func.fmtgs(f, info)
            }
            Self::Block(b) => b.fmtgs(f, info),
            Self::If(condition, yes, no) => {
                write!(f, "if ")?;
                condition.fmtgs(f, info)?;
                write!(f, " ")?;
                yes.fmtgs(f, info)?;
                if let Some(no) = no {
                    write!(f, " else ")?;
                    no.fmtgs(f, info)?;
                }
                Ok(())
            }
            Self::Loop(b) => {
                write!(f, "loop ")?;
                b.fmtgs(f, info)
            }
            Self::For(var, i, b) => {
                write!(f, "for {} ", var)?;
                i.fmtgs(f, info)?;
                write!(f, " ")?;
                b.fmtgs(f, info)
            }
            Self::Switch(var, arms, force) => {
                if *force {
                    writeln!(f, "switch! {var} {{")?;
                } else {
                    writeln!(f, "switch {var} {{")?;
                }
                for (t, action) in arms {
                    t.fmtgs(f, info)?;
                    write!(f, " ")?;
                    action.fmtgs(f, info)?;
                    writeln!(f)?;
                }
                write!(f, "}}")
            }
            Self::Match(var, arms) => {
                write!(f, "match {var} {{")?;
                for (condition, action) in arms {
                    condition.fmtgs(f, info)?;
                    write!(f, " ")?;
                    action.fmtgs(f, info)?;
                    writeln!(f)?;
                }
                write!(f, "}}")
            }
            Self::IndexFixed(statement, index) => {
                statement.fmtgs(f, info)?;
                write!(f, ".{index}")
            }
            Self::EnumVariant(variant, inner) => {
                write!(f, "{variant}: ")?;
                inner.fmtgs(f, info)
            }
            Self::TypeDefinition(name, t) => write!(f, "type {name} {t}"),
            Self::Macro(m) => {
                write!(f, "!({m})")
            }
        }
    }
}
impl Display for SStatementEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None)
    }
}

impl SStatement {
    pub fn fmtgs(&self, f: &mut Formatter, info: Option<&GlobalScriptInfo>) -> fmt::Result {
        if let Some((opt, derefs)) = &self.output_to {
            if let Some(forced_type) = &self.force_output_type {
                write!(f, "{}{}::", "*".repeat(*derefs), opt)?;
                forced_type.fmtgs(f, info)?;
                write!(f, " = ")?;
            } else {
                write!(f, "{}{} = ", "*".repeat(*derefs), opt)?;
            }
        }
        self.statement.fmtgs(f, info)
    }
}
impl Display for SStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None)
    }
}

impl SFunction {
    pub fn fmtgs(&self, f: &mut Formatter, info: Option<&GlobalScriptInfo>) -> fmt::Result {
        write!(f, "(")?;
        for (i, (name, t)) in self.inputs.iter().enumerate() {
            if i > 0 {
                write!(f, " {name}")?;
            } else {
                write!(f, "{name} ")?;
            }
            t.fmtgs(f, info)?;
        }
        write!(f, ") ")?;
        self.block.fmtgs(f, info)
    }
}

impl SBlock {
    pub fn fmtgs(&self, f: &mut Formatter, info: Option<&GlobalScriptInfo>) -> fmt::Result {
        match self.statements.len() {
            0 => write!(f, "{{}}"),
            1 => self.statements[0].fmtgs(f, info),
            _ => {
                writeln!(f, "{{")?;
                for statement in self.statements.iter() {
                    statement.fmtgs(f, info)?;
                    writeln!(f)?;
                }
                write!(f, "}}")
            }
        }
    }
}
