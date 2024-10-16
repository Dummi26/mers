use std::fmt::{self, Display, Formatter};

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
    For(SStatement, SStatement, SStatement),
    Switch(SStatement, Vec<(VType, SStatement, SStatement)>, bool),
    Match(Vec<(SStatement, SStatement, SStatement)>),
    IndexFixed(SStatement, usize),
    EnumVariant(String, SStatement),
    TypeDefinition(String, VType),
    Macro(Macro),
}
impl SStatementEnum {
    pub fn to(self) -> SStatement {
        SStatement::new(self)
    }
}

#[derive(Debug)]
pub struct SStatement {
    pub derefs: usize,
    /// if the statement is a Variable that doesn't exist yet, it will be initialized.
    /// if it's a variable that exists, but is_ref is false, an error may show up: cannot dereference
    /// if the third value is true, the variable will always be initialized, shadowing previous mentions of the same name.
    pub output_to: Option<(Box<SStatement>, bool)>,
    pub statement: Box<SStatementEnum>,
    pub force_output_type: Option<VType>,
}

impl SStatement {
    pub fn new(statement: SStatementEnum) -> Self {
        Self {
            derefs: 0,
            output_to: None,
            statement: Box::new(statement),
            force_output_type: None,
        }
    }
    pub fn output_to(mut self, statement: SStatement) -> Self {
        self.output_to = Some((Box::new(statement), false));
        self
    }
    /// like output_to, but always initializes the variable (shadows previous variables of the same name)
    pub fn initialize_to(mut self, statement: SStatement) -> Self {
        self.output_to = Some((Box::new(statement), true));
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
    pub statement: SStatement,
}
impl SFunction {
    pub fn new(inputs: Vec<(String, VType)>, statement: SStatement) -> Self {
        Self { inputs, statement }
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
                for (_i, v) in v.iter().enumerate() {
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
                        write!(f, " ")?;
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
            Self::For(assign_to, i, b) => {
                write!(f, "{} ", form.loop_for(info, "for".to_owned()))?;
                assign_to.fmtgs(f, info, form, file)?;
                write!(f, " ")?;
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
                for (t, assign_to, action) in arms {
                    write!(f, "{}", form.line_prefix())?;
                    t.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                    assign_to.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                    action.fmtgs(f, info, form, file)?;
                    writeln!(f)?;
                }
                form.go_shallower();
                write!(f, "{}", form.line_prefix())?;
                write!(f, "{}", form.close_bracket(info, "}".to_owned()))
            }
            Self::Match(arms) => {
                write!(
                    f,
                    "{} {}",
                    form.kw_match(info, "match".to_owned()),
                    form.open_bracket(info, "{".to_owned())
                )?;
                form.go_deeper();
                for (condition, assign_to, action) in arms {
                    write!(f, "{}", form.line_prefix())?;
                    condition.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                    assign_to.fmtgs(f, info, form, file)?;
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
        // output output_to
        if let Some((opt, is_init)) = &self.output_to {
            write!(
                f,
                "{} {} ",
                opt.with(info, file),
                if *is_init { ":=" } else { "=" }
            )?;
        }
        // output self
        if let Some(force_opt) = &self.force_output_type {
            write!(f, "-> ")?;
            force_opt.fmtgs(f, info, form, file)?;
            write!(f, " ")?;
        }
        write!(f, "{}", "*".repeat(self.derefs))?;
        self.statement.fmtgs(f, info, form, file)?;
        write!(f, ",")
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
                write!(f, " {name} ")?;
            } else {
                write!(f, "{name} ")?;
            }
            t.fmtgs(f, info, form, file)?;
        }
        write!(f, ") ")?;
        self.statement.fmtgs(f, info, form, file)
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
