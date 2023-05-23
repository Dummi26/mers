use std::fmt::{self, Display, Formatter, Pointer};

use crate::lang::global_info::ColorFormatMode;

use super::{
    code_macro::Macro, fmtgs::FormatGs, global_info::GlobalScriptInfo, val_data::VData,
    val_type::VType,
};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SStatement {
    /// if the statement is a Variable that doesn't exist yet, it will be initialized.
    /// if it's a variable that exists, but is_ref is false, an error may show up: cannot dereference
    /// NOTE: Maybe add a bool that indicates a variable should be newly declared, shadowing old ones with the same name.
    pub output_to: Option<(Box<SStatement>, usize)>,
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
    pub fn output_to(mut self, statement: SStatement, derefs: usize) -> Self {
        self.output_to = Some((Box::new(statement), derefs));
        self
    }
    // forces the statement's output to fit in a certain type.
    pub fn force_output_type(mut self, force_output_type: Option<VType>) -> Self {
        self.force_output_type = force_output_type;
        self
    }
}

/// A block of code is a collection of statements.
#[derive(Debug)]
pub struct SBlock {
    pub statements: Vec<SStatement>,
}
impl SBlock {
    pub fn new(statements: Vec<SStatement>) -> Self {
        Self { statements }
    }
}

// A function is a block of code that starts with some local variables as inputs and returns some value as its output. The last statement in the block will be the output.
#[derive(Debug)]
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

impl FormatGs for SStatementEnum {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        match self {
            Self::Value(v) => v.fmtgs(f, info, form, file),
            Self::Tuple(v) => {
                write!(f, "{}", form.open_bracket(info, "[".to_owned()))?;
                for (i, v) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    v.fmtgs(f, info, form, file)?;
                }
                write!(f, "{}", form.close_bracket(info, "]".to_owned()))
            }
            Self::List(v) => {
                write!(f, "{}", form.open_bracket(info, "[".to_owned()))?;
                for (i, v) in v.iter().enumerate() {
                    v.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                }
                write!(f, "{}", form.close_bracket(info, "...]".to_owned()))
            }
            Self::Variable(var, reference) => {
                if *reference {
                    write!(f, "{}", form.variable_ref_symbol(info, "&".to_owned()))?;
                }
                write!(f, "{}", form.variable(info, var.to_owned()))
            }
            Self::FunctionCall(func, args) => {
                write!(
                    f,
                    "{}{}",
                    form.fncall(info, func.to_owned()),
                    form.open_bracket(info, "(".to_owned())
                )?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ");
                    }
                    arg.fmtgs(f, info, form, file)?;
                }
                write!(f, "{}", form.close_bracket(info, ")".to_owned()))
            }
            Self::FunctionDefinition(name, func) => {
                if let Some(name) = name {
                    write!(
                        f,
                        "{} {}",
                        form.fndef_fn(info, "fn".to_owned()),
                        form.fndef_name(info, name.to_owned())
                    )?;
                }
                func.fmtgs(f, info, form, file)
            }
            Self::Block(b) => b.fmtgs(f, info, form, file),
            Self::If(condition, yes, no) => {
                write!(f, "{} ", form.if_if(info, "if".to_owned()))?;
                condition.fmtgs(f, info, form, file)?;
                write!(f, " ")?;
                yes.fmtgs(f, info, form, file)?;
                if let Some(no) = no {
                    write!(f, " {} ", form.if_else(info, "else".to_owned()))?;
                    no.fmtgs(f, info, form, file)?;
                }
                Ok(())
            }
            Self::Loop(b) => {
                write!(f, "{} ", form.loop_loop(info, "loop".to_owned()))?;
                b.fmtgs(f, info, form, file)
            }
            Self::For(var, i, b) => {
                write!(f, "{} {} ", form.loop_for(info, "for".to_owned()), var)?;
                i.fmtgs(f, info, form, file)?;
                write!(f, " ")?;
                b.fmtgs(f, info, form, file)
            }
            Self::Switch(var, arms, force) => {
                if *force {
                    writeln!(
                        f,
                        "{} {var} {}",
                        form.kw_switch(info, "switch!".to_owned()),
                        form.open_bracket(info, "{".to_owned())
                    )?;
                } else {
                    writeln!(
                        f,
                        "{} {var} {}",
                        form.kw_switch(info, "switch".to_owned()),
                        form.open_bracket(info, "{".to_owned())
                    )?;
                }
                form.go_deeper();
                for (t, action) in arms {
                    write!(f, "{}", form.line_prefix())?;
                    t.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                    action.fmtgs(f, info, form, file)?;
                    writeln!(f)?;
                }
                form.go_shallower();
                write!(f, "{}", form.line_prefix())?;
                write!(f, "{}", form.close_bracket(info, "}".to_owned()))
            }
            Self::Match(var, arms) => {
                write!(
                    f,
                    "{} {var} {}",
                    form.kw_match(info, "match".to_owned()),
                    form.open_bracket(info, "{".to_owned())
                )?;
                form.go_deeper();
                for (condition, action) in arms {
                    write!(f, "{}", form.line_prefix())?;
                    condition.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                    action.fmtgs(f, info, form, file)?;
                    writeln!(f)?;
                }
                form.go_shallower();
                write!(f, "{}", form.line_prefix())?;
                write!(f, "{}", form.close_bracket(info, "}".to_owned()))
            }
            Self::IndexFixed(statement, index) => {
                statement.fmtgs(f, info, form, file)?;
                write!(f, ".{index}")
            }
            Self::EnumVariant(variant, inner) => {
                write!(f, "{variant}: ")?;
                inner.fmtgs(f, info, form, file)
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
        self.fmtgs(f, None, &mut super::fmtgs::FormatInfo::default(), None)
    }
}

impl FormatGs for SStatement {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        if let Some((opt, derefs)) = &self.output_to {
            // TODO!
            match opt.statement.as_ref() {
                // SStatementEnum::Variable(name, is_ref) => {
                //     let derefs = if !is_ref { *derefs + 1 } else { *derefs };
                //     write!(
                //         f,
                //         "{}{} = ",
                //         "*".repeat(derefs),
                //         SStatementEnum::Variable(name.to_owned(), false).with(info, file)
                //     )?;
                // }
                _ => {
                    write!(f, "{}{} = ", "*".repeat(*derefs), opt.with(info, file))?;
                }
            }
        }
        if let Some(force_opt) = &self.force_output_type {
            write!(f, " -> ")?;
            force_opt.fmtgs(f, info, form, file)?;
        }
        self.statement.fmtgs(f, info, form, file)
    }
}
impl Display for SStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None, &mut super::fmtgs::FormatInfo::default(), None)
    }
}

impl FormatGs for SFunction {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, (name, t)) in self.inputs.iter().enumerate() {
            if i > 0 {
                write!(f, " {name}")?;
            } else {
                write!(f, "{name} ")?;
            }
            t.fmtgs(f, info, form, file)?;
        }
        write!(f, ") ")?;
        self.block.fmtgs(f, info, form, file)
    }
}

impl FormatGs for SBlock {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        match self.statements.len() {
            0 => write!(f, "{{}}"),
            // 1 => self.statements[0].fmtgs(f, info, form, file),
            _ => {
                writeln!(f, "{}", form.open_bracket(info, "{".to_owned()))?;
                form.go_deeper();
                for statement in self.statements.iter() {
                    write!(f, "{}", form.line_prefix())?;
                    statement.fmtgs(f, info, form, file)?;
                    writeln!(f)?;
                }
                form.go_shallower();
                write!(f, "{}", form.line_prefix())?;
                write!(f, "{}", form.close_bracket(info, "}".to_owned()))
            }
        }
    }
}
