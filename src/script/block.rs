// A code block is any section of code. It contains its own local variables and functions, as well as a list of statements.
// Types starting with S are directly parsed from Strings and unchecked. Types starting with T are type-checked templates for R-types. Types starting with R are runnable. S are converted to T after parsing is done, and T are converted to R whenever they need to run.

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use self::to_runnable::ToRunnableError;

use super::{
    builtins::BuiltinFunction,
    value::{VData, VDataEnum, VSingleType, VType},
};

// Represents a block of code
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
    inputs: Vec<(String, VType)>,
    block: SBlock,
}
impl SFunction {
    pub fn new(inputs: Vec<(String, VType)>, block: SBlock) -> Self {
        Self { inputs, block }
    }
}

#[derive(Debug)]
pub struct SStatement {
    pub output_to: Option<String>,
    pub statement: Box<SStatementEnum>,
}
impl SStatement {
    pub fn new(statement: SStatementEnum) -> Self {
        Self {
            output_to: None,
            statement: Box::new(statement),
        }
    }
    pub fn output_to(mut self, var: String) -> Self {
        self.output_to = Some(var);
        self
    }
}

#[derive(Debug)]
pub enum SStatementEnum {
    Value(VData),
    Tuple(Vec<SStatement>),
    List(Vec<SStatement>),
    Variable(String),
    FunctionCall(String, Vec<SStatement>),
    FunctionDefinition(Option<String>, SFunction),
    Block(SBlock),
    If(SStatement, SStatement, Option<SStatement>),
    While(SStatement),
    For(String, SStatement, SStatement),
    Switch(String, Vec<(VType, SStatement)>, bool),
    // Match(???),
    IndexFixed(SStatement, usize),
}
impl Into<SStatement> for SStatementEnum {
    fn into(self) -> SStatement {
        SStatement::new(self)
    }
}

// Conversion

type Am<T> = Arc<Mutex<T>>;
fn am<T>(i: T) -> Am<T> {
    Arc::new(Mutex::new(i))
}

pub fn to_runnable(f: SFunction) -> Result<RScript, ToRunnableError> {
    to_runnable::to_runnable(f)
}

pub mod to_runnable {
    use std::{
        collections::HashMap,
        fmt::{Debug, Display},
        sync::Arc,
    };

    use crate::script::value::{VDataEnum, VSingleType, VType};

    use super::{
        BuiltinFunction, RBlock, RFunction, RScript, RStatement, RStatementEnum, SBlock, SFunction,
        SStatement, SStatementEnum,
    };

    pub enum ToRunnableError {
        MainWrongInput,
        UseOfUndefinedVariable(String),
        UseOfUndefinedFunction(String),
        FunctionWrongArgCount(String, usize, usize),
        InvalidType {
            expected: VType,
            found: VType,
            problematic: Vec<VSingleType>,
        },
        InvalidTypeForWhileLoop(VType),
        CaseForceButTypeNotCovered(VType),
        NotIndexableFixed(VType, usize),
    }
    impl Debug for ToRunnableError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self}")
        }
    }
    impl Display for ToRunnableError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MainWrongInput => write!(
                    f,
                    "Main function had the wrong input. This is a bug and should never happen."
                ),
                Self::UseOfUndefinedVariable(v) => write!(f, "Cannot use variable \"{v}\" as it isn't defined (yet?)."),
                Self::UseOfUndefinedFunction(v) => write!(f, "Cannot use function \"{v}\" as it isn't defined (yet?)."),
                Self::FunctionWrongArgCount(v, a, b) => write!(f, "Tried to call function \"{v}\", which takes {a} arguments, with {b} arguments instead."),
                Self::InvalidType {
                    expected,
                    found,
                    problematic,
                } => {
                    write!(f, "Invalid type: Expected {expected:?} but found {found:?}, which includes {problematic:?}, which is not covered.")
                }
                Self::InvalidTypeForWhileLoop(v) => write!(f, "Invalid type: Expected bool or Tuples of length 0 or 1 as return types for the while loop, but found {v:?} instead."),
                Self::CaseForceButTypeNotCovered(v) => write!(f, "Switch! statement, but not all types covered. Types to cover: {v}"),
                Self::NotIndexableFixed(t, i) => write!(f, "Cannot use fixed-index {i} on type {t}."),
            }
        }
    }

    // Global, shared between all
    struct GInfo {
        vars: usize,
    }
    // Local, used to keep local variables separated
    #[derive(Clone)]
    struct LInfo {
        vars: HashMap<String, (usize, VType)>,
        fns: HashMap<String, Arc<RFunction>>,
    }

    pub fn to_runnable(s: SFunction) -> Result<RScript, ToRunnableError> {
        if s.inputs.len() != 1 || s.inputs[0].0 != "args" {
            return Err(ToRunnableError::MainWrongInput);
        }
        if s.inputs[0].1
            != (VType {
                types: vec![VSingleType::List(VType {
                    types: vec![VSingleType::String],
                })],
            })
        {}
        let mut ginfo = GInfo { vars: 0 };
        let func = function(
            &s,
            &mut ginfo,
            LInfo {
                vars: HashMap::new(),
                fns: HashMap::new(),
            },
        )?;
        Ok(RScript::new(func, ginfo.vars)?)
    }

    // go over every possible known-type input for the given function, returning all possible RFunctions.
    fn get_all_functions(
        s: &SFunction,
        ginfo: &mut GInfo,
        linfo: &mut LInfo,
        input_vars: &Vec<usize>,
        inputs: &mut Vec<VSingleType>,
        out: &mut Vec<(Vec<VSingleType>, VType)>,
    ) -> Result<(), ToRunnableError> {
        if s.inputs.len() > inputs.len() {
            let input_here = &s.inputs[inputs.len()].1;
            for t in &input_here.types {
                inputs.push(t.clone().into());
                get_all_functions(s, ginfo, linfo, input_vars, inputs, out)?;
                inputs.pop();
            }
            Ok(())
        } else {
            // set the types
            for (varid, vartype) in s.inputs.iter().zip(inputs.iter()) {
                linfo.vars.get_mut(&varid.0).unwrap().1 = vartype.clone().into();
            }
            out.push((inputs.clone(), block(&s.block, ginfo, linfo.clone())?.out()));
            Ok(())
        }
    }
    fn function(
        s: &SFunction,
        ginfo: &mut GInfo,
        mut linfo: LInfo,
    ) -> Result<RFunction, ToRunnableError> {
        let mut input_vars = vec![];
        let mut input_types = vec![];
        for (iname, itype) in &s.inputs {
            linfo
                .vars
                .insert(iname.clone(), (ginfo.vars, itype.clone()));
            input_vars.push(ginfo.vars);
            input_types.push(itype.clone());
            ginfo.vars += 1;
        }
        let mut all_outs = vec![];
        get_all_functions(
            s,
            ginfo,
            &mut linfo,
            &input_vars,
            &mut Vec::with_capacity(s.inputs.len()),
            &mut all_outs,
        )?;
        // set the types to all possible types (get_all_functions sets the types to one single type to get the return type of the block for that case)
        for (varid, vartype) in s.inputs.iter().zip(input_types.iter()) {
            linfo.vars.get_mut(&varid.0).unwrap().1 = vartype.clone();
        }
        Ok(RFunction {
            inputs: input_vars,
            input_types,
            input_output_map: all_outs,
            block: block(&s.block, ginfo, linfo)?,
        })
    }

    fn block(s: &SBlock, ginfo: &mut GInfo, mut linfo: LInfo) -> Result<RBlock, ToRunnableError> {
        let mut statements = Vec::new();
        for st in &s.statements {
            statements.push(statement(st, ginfo, &mut linfo)?);
        }
        Ok(RBlock { statements })
    }

    fn statement(
        s: &SStatement,
        ginfo: &mut GInfo,
        linfo: &mut LInfo,
    ) -> Result<RStatement, ToRunnableError> {
        let mut statement = match &*s.statement {
            SStatementEnum::Value(v) => RStatementEnum::Value(v.clone()),
            SStatementEnum::Tuple(v) | SStatementEnum::List(v) => {
                let mut w = Vec::with_capacity(v.len());
                for v in v {
                    w.push(statement(v, ginfo, linfo)?);
                }
                if let SStatementEnum::List(_) = &*s.statement {
                    RStatementEnum::List(w)
                } else {
                    RStatementEnum::Tuple(w)
                }
            }
            SStatementEnum::Variable(v) => {
                if let Some(var) = linfo.vars.get(v) {
                    RStatementEnum::Variable(var.0, var.1.clone())
                } else {
                    return Err(ToRunnableError::UseOfUndefinedVariable(v.clone()));
                }
            }
            SStatementEnum::FunctionCall(v, args) => {
                let mut rargs = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    rargs.push(statement(arg, ginfo, linfo)?);
                }
                if let Some(func) = linfo.fns.get(v) {
                    if rargs.len() != func.inputs.len() {
                        return Err(ToRunnableError::FunctionWrongArgCount(
                            v.clone(),
                            func.inputs.len(),
                            rargs.len(),
                        ));
                    }
                    for (i, rarg) in rargs.iter().enumerate() {
                        let rarg = rarg.out();
                        let out = rarg.fits_in(&func.input_types[i]);
                        if !out.is_empty() {
                            return Err(ToRunnableError::InvalidType {
                                expected: func.input_types[i].clone(),
                                found: rarg,
                                problematic: out,
                            });
                        }
                    }
                    RStatementEnum::FunctionCall(func.clone(), rargs)
                } else {
                    // TODO: type-checking for builtins
                    if let Some(builtin) = BuiltinFunction::get(v) {
                        RStatementEnum::BuiltinFunction(builtin, rargs)
                    } else {
                        return Err(ToRunnableError::UseOfUndefinedFunction(v.clone()));
                    }
                }
            }
            SStatementEnum::FunctionDefinition(name, f) => {
                if let Some(name) = name {
                    // named function => add to global functions
                    linfo
                        .fns
                        .insert(name.clone(), Arc::new(function(f, ginfo, linfo.clone())?));
                    RStatementEnum::Value(VDataEnum::Tuple(vec![]).to())
                } else {
                    // anonymous function => return as value
                    RStatementEnum::Value(
                        VDataEnum::Function(function(f, ginfo, linfo.clone())?).to(),
                    )
                }
            }
            SStatementEnum::Block(b) => RStatementEnum::Block(block(&b, ginfo, linfo.clone())?),
            SStatementEnum::If(c, t, e) => RStatementEnum::If(
                {
                    let condition = statement(&c, ginfo, linfo)?;
                    let out = condition.out().fits_in(&VType {
                        types: vec![VSingleType::Bool],
                    });
                    if out.is_empty() {
                        condition
                    } else {
                        return Err(ToRunnableError::InvalidType {
                            expected: VSingleType::Bool.into(),
                            found: condition.out(),
                            problematic: out,
                        });
                    }
                },
                statement(&t, ginfo, linfo)?,
                match e {
                    Some(v) => Some(statement(&v, ginfo, linfo)?),
                    None => None,
                },
            ),
            SStatementEnum::While(c) => RStatementEnum::While({
                let condition = statement(&c, ginfo, linfo)?;
                let out = condition.out();
                let out1 = out.fits_in(&VSingleType::Bool.into());
                if out1.is_empty() {
                    condition
                } else {
                    if out.types.is_empty() {
                        return Err(ToRunnableError::InvalidTypeForWhileLoop(out));
                    }
                    for t in out.types.iter() {
                        if let VSingleType::Tuple(v) = t {
                            if v.len() > 1 {
                                return Err(ToRunnableError::InvalidTypeForWhileLoop(out));
                            }
                        } else {
                            return Err(ToRunnableError::InvalidTypeForWhileLoop(out));
                        }
                    }
                    condition
                }
            }),
            SStatementEnum::For(v, c, b) => {
                let mut linfo = linfo.clone();
                let container = statement(&c, ginfo, &mut linfo)?;
                linfo
                    .vars
                    .insert(v.clone(), (ginfo.vars, container.out().inner_types()));
                let for_loop_var = ginfo.vars;
                ginfo.vars += 1;
                let block = statement(&b, ginfo, &mut linfo)?;
                let o = RStatementEnum::For(for_loop_var, container, block);
                o
            }
            SStatementEnum::Switch(switch_on, cases, force) => {
                if let Some(switch_on_v) = linfo.vars.get(switch_on).cloned() {
                    let mut ncases = Vec::with_capacity(cases.len());
                    let og_type = linfo.vars.get(switch_on).unwrap().1.clone();
                    for case in cases {
                        linfo.vars.get_mut(switch_on).unwrap().1 = case.0.clone();
                        ncases.push((case.0.clone(), statement(&case.1, ginfo, linfo)?));
                    }
                    linfo.vars.get_mut(switch_on).unwrap().1 = og_type;

                    let switch_on_out = switch_on_v.1;
                    if *force {
                        for val_type in switch_on_out.types.iter() {
                            let val_type: VType = val_type.clone().into();
                            let mut linf2 = linfo.clone();
                            linf2.vars.get_mut(switch_on).unwrap().1 = val_type.clone();
                            'force: {
                                for (case_type, _) in cases {
                                    if val_type.fits_in(&case_type).is_empty() {
                                        break 'force;
                                    }
                                }
                                return Err(ToRunnableError::CaseForceButTypeNotCovered(val_type));
                            }
                        }
                    }
                    RStatementEnum::Switch(
                        RStatementEnum::Variable(switch_on_v.0, switch_on_out).to(),
                        ncases,
                    )
                } else {
                    return Err(ToRunnableError::UseOfUndefinedVariable(switch_on.clone()));
                }
            }
            SStatementEnum::IndexFixed(st, i) => {
                let st = statement(st, ginfo, linfo)?;
                let ok = 'ok: {
                    let mut one = false;
                    for t in st.out().types {
                        one = true;
                        // only if all types are indexable by i
                        match t {
                            VSingleType::Tuple(v) => {
                                if v.len() <= *i {
                                    break 'ok false;
                                }
                            }
                            _ => break 'ok false,
                        }
                    }
                    one
                };
                if ok {
                    RStatementEnum::IndexFixed(st, *i)
                } else {
                    return Err(ToRunnableError::NotIndexableFixed(st.out(), *i));
                }
            }
        }
        .to();
        if let Some(opt) = &s.output_to {
            if let Some(var) = linfo.vars.get(opt) {
                let out = statement.out();
                let var_id = var.0;
                let var_out = &var.1;
                let inv_types = out.fits_in(&var_out);
                if !inv_types.is_empty() {
                    eprintln!("Warn: shadowing variable {opt} because statement's output type {out} does not fit in the original variable's {var_out}. This might become an error in the future, or it might stop shadowing the variiable entirely - for stable scripts, avoid this by giving the variable a different name.");
                    linfo.vars.insert(opt.clone(), (ginfo.vars, out));
                    statement.output_to = Some(ginfo.vars);
                    ginfo.vars += 1;
                } else {
                    statement.output_to = Some(var_id);
                }
            } else {
                linfo
                    .vars
                    .insert(opt.clone(), (ginfo.vars, statement.out()));
                statement.output_to = Some(ginfo.vars);
                ginfo.vars += 1;
            }
        }
        Ok(statement)
    }
}

// Runnable

#[derive(Clone, Debug)]
pub struct RBlock {
    statements: Vec<RStatement>,
}
impl RBlock {
    pub fn run(&self, vars: &Vec<Am<VData>>) -> VData {
        let mut last = None;
        for statement in &self.statements {
            last = Some(statement.run(vars));
        }
        if let Some(v) = last {
            v
        } else {
            VDataEnum::Tuple(vec![]).to()
        }
    }
    pub fn out(&self) -> VType {
        if let Some(last) = self.statements.last() {
            last.out()
        } else {
            VType {
                types: vec![VSingleType::Tuple(vec![])],
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RFunction {
    pub inputs: Vec<usize>,
    pub input_types: Vec<VType>,
    pub input_output_map: Vec<(Vec<VSingleType>, VType)>,
    pub block: RBlock,
}
impl RFunction {
    pub fn run(&self, vars: &Vec<Am<VData>>) -> VData {
        self.block.run(vars)
    }
    pub fn out(&self, input_types: &Vec<VSingleType>) -> VType {
        self.input_output_map
            .iter()
            .find_map(|v| {
                if v.0 == *input_types {
                    Some(v.1.clone())
                } else {
                    None
                }
            })
            .expect("invalid args for function! possible issue with type-checker if this can be reached! feel free to report a bug.")
    }
    pub fn out_all(&self) -> VType {
        self.block.out()
    }
    pub fn in_types(&self) -> &Vec<VType> {
        &self.input_types
    }
}

#[derive(Clone, Debug)]
pub struct RStatement {
    output_to: Option<usize>,
    statement: Box<RStatementEnum>,
}
impl RStatement {
    pub fn run(&self, vars: &Vec<Am<VData>>) -> VData {
        let out = self.statement.run(vars);
        if let Some(v) = self.output_to {
            *vars[v].lock().unwrap() = out;
            VDataEnum::Tuple(vec![]).to()
        } else {
            out
        }
    }
    pub fn out(&self) -> VType {
        if self.output_to.is_some() {
            return VType {
                types: vec![VSingleType::Tuple(vec![])],
            };
        }
        self.statement.out()
    }
}

#[derive(Clone, Debug)]
pub enum RStatementEnum {
    Value(VData),
    Tuple(Vec<RStatement>),
    List(Vec<RStatement>),
    Variable(usize, VType), // Arc<Mutex<..>> here, because imagine variable in for loop that is used in a different thread -> we need multiple "same" variables
    FunctionCall(Arc<RFunction>, Vec<RStatement>),
    BuiltinFunction(BuiltinFunction, Vec<RStatement>),
    Block(RBlock),
    If(RStatement, RStatement, Option<RStatement>),
    While(RStatement),
    For(usize, RStatement, RStatement),
    Switch(RStatement, Vec<(VType, RStatement)>),
    IndexFixed(RStatement, usize),
}
impl RStatementEnum {
    pub fn run(&self, vars: &Vec<Am<VData>>) -> VData {
        match self {
            Self::Value(v) => v.clone(),
            Self::Tuple(v) => {
                let mut w = vec![];
                for v in v {
                    w.push(v.run(vars));
                }
                VDataEnum::Tuple(w).to()
            }
            Self::List(v) => {
                let mut w = vec![];
                let mut out = VType { types: vec![] };
                for v in v {
                    let val = v.run(vars);
                    out = out | val.out();
                    w.push(val);
                }
                VDataEnum::List(out, w).to()
            }
            Self::Variable(v, _) => vars[*v].lock().unwrap().clone(),
            Self::FunctionCall(func, args) => {
                for (i, input) in func.inputs.iter().enumerate() {
                    *vars[*input].lock().unwrap() = args[i].run(vars);
                }
                func.run(vars)
            }
            Self::Block(b) => b.run(vars),
            Self::If(c, t, e) => {
                if let VDataEnum::Bool(v) = c.run(vars).data {
                    if v {
                        t.run(vars)
                    } else {
                        if let Some(e) = e {
                            e.run(vars)
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            Self::While(c) => loop {
                // While loops blocks can return a bool (false to break from the loop) or a 0-1 length tuple (0-length => continue, 1-length => break with value)
                match c.run(vars).data {
                    VDataEnum::Bool(v) => {
                        if !v {
                            break VDataEnum::Tuple(vec![]).to();
                        }
                    }
                    VDataEnum::Tuple(v) if v.len() == 1 => break v[0].clone(),
                    _ => unreachable!(),
                }
            },
            Self::For(v, c, b) => {
                let c = c.run(vars);
                let mut vars = vars.clone();
                let mut in_loop = |c| {
                    vars[*v] = Arc::new(Mutex::new(c));
                    b.run(&vars);
                };
                match c.data {
                    VDataEnum::Int(v) => {
                        for i in 0..v {
                            in_loop(VDataEnum::Int(i).to());
                        }
                    }
                    VDataEnum::String(v) => {
                        for ch in v.chars() {
                            in_loop(VDataEnum::String(ch.to_string()).to())
                        }
                    }
                    VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                        for v in v {
                            in_loop(v)
                        }
                    }
                    _ => unreachable!(),
                }
                VDataEnum::Tuple(vec![]).to()
            }
            Self::BuiltinFunction(v, args) => v.run(args, vars),
            Self::Switch(switch_on, cases) => {
                let switch_on = switch_on.run(vars);
                let switch_on_type = switch_on.out();
                let mut out = VDataEnum::Tuple(vec![]).to();
                for (case_type, case_action) in cases.iter() {
                    if switch_on_type.fits_in(case_type).is_empty() {
                        out = case_action.run(vars);
                        break;
                    }
                }
                out
            }
            Self::IndexFixed(st, i) => st.run(vars).get(*i).unwrap(),
        }
    }
    pub fn out(&self) -> VType {
        match self {
            Self::Value(v) => v.out(),
            Self::Tuple(v) | Self::List(v) => {
                VSingleType::Tuple(v.iter().map(|v| v.out()).collect()).into()
            }
            Self::Variable(_, t) => t.clone(),
            Self::FunctionCall(f, _) => {
                eprintln!("Warn: generalizing a functions return type regardless of the inputs. Type-checker might assume this value can have more types than it really can.");
                f.out_all()
            }
            Self::Block(b) => b.out(),
            Self::If(_, a, b) => {
                if let Some(b) = b {
                    a.out() | b.out()
                } else {
                    a.out()
                }
            }
            Self::While(c) => todo!("while loop output type"),
            Self::For(_, _, b) => {
                // returns the return value from the last iteration or nothing if there was no iteration
                b.out()
                    | VType {
                        types: vec![VSingleType::Tuple(vec![])],
                    }
            }
            Self::BuiltinFunction(f, _) => f.returns(),
            Self::Switch(_, cases) => {
                let mut out = VSingleType::Tuple(vec![]).into(); // if nothing is executed
                for (_, case) in cases.iter() {
                    out = out | case.out();
                }
                out
            }
            Self::IndexFixed(st, i) => st.out().get(*i).unwrap(),
        }
    }
    pub fn to(self) -> RStatement {
        RStatement {
            output_to: None,
            statement: Box::new(self),
        }
    }
}

#[derive(Debug)]
pub struct RScript {
    main: RFunction,
    vars: usize,
}
impl RScript {
    fn new(main: RFunction, vars: usize) -> Result<Self, ToRunnableError> {
        if main.inputs.len() != 1 {
            return Err(ToRunnableError::MainWrongInput);
        }
        Ok(Self { main, vars })
    }
    pub fn run(&self, args: Vec<String>) -> VData {
        let mut vars = Vec::with_capacity(self.vars);
        vars.push(am(VDataEnum::List(
            VSingleType::String.into(),
            args.into_iter()
                .map(|v| VDataEnum::String(v).to())
                .collect(),
        )
        .to()));
        for _i in 1..self.vars {
            vars.push(am(VDataEnum::Tuple(vec![]).to()));
        }
        self.main.run(&vars)
    }
}

impl Display for SFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, (n, t)) in self.inputs.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{n} {t}")?;
        }
        write!(f, ") {}", self.block)?;
        Ok(())
    }
}
impl Display for SBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for s in self.statements.iter() {
            writeln!(f, "{s}")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
impl Display for VType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, t) in self.types.iter().enumerate() {
            if i != 0 {
                write!(f, "/")?;
            }
            write!(f, "{t}")?;
        }
        Ok(())
    }
}
impl Display for VSingleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),
            Self::Tuple(types) => {
                write!(f, "[")?;
                for (i, t) in types.iter().enumerate() {
                    if i != 0 {
                        write!(f, " {t}")?;
                    } else {
                        write!(f, "{t}")?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            Self::List(t) => write!(f, "[{t}]"),
            Self::Function(args, out) => write!(f, "({args:?}) -> {out}"),
            Self::Thread(_) => write!(f, "THREAD"),
        }
    }
}
impl Display for SStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(to) = self.output_to.as_ref() {
            write!(f, "{} = ", to.as_str())?;
        }
        write!(f, "{}", self.statement)
    }
}
impl Display for SStatementEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SStatementEnum::Value(v) => write!(f, "{v}"),
            SStatementEnum::Tuple(v) => {
                write!(
                    f,
                    "[{}]",
                    v.iter().map(|v| format!("{} ", v)).collect::<String>()
                )
            }
            SStatementEnum::List(v) => {
                write!(
                    f,
                    "[{} ...]",
                    v.iter().map(|v| format!("{} ", v)).collect::<String>()
                )
            }
            SStatementEnum::Variable(v) => write!(f, "{v}"),
            SStatementEnum::FunctionCall(func, args) => {
                write!(f, "{func}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            SStatementEnum::FunctionDefinition(name, func) => {
                if let Some(name) = name {
                    write!(f, "{name}{func}")
                } else {
                    write!(f, "{func}")
                }
            }
            SStatementEnum::Block(b) => write!(f, "{b}"),
            SStatementEnum::If(c, a, b) => {
                write!(f, "if {c} {a}")?;
                if let Some(b) = b {
                    write!(f, " else {b}")?;
                }
                Ok(())
            }
            SStatementEnum::While(c) => write!(f, "while {c}"),
            SStatementEnum::For(v, c, b) => write!(f, "for {v} {c} {b}"),
            SStatementEnum::Switch(switch_on, cases, force) => {
                writeln!(
                    f,
                    "switch{} {} {{",
                    if *force { "!" } else { "" },
                    switch_on
                )?;
                for (case_type, case_action) in cases.iter() {
                    writeln!(f, "{} {}", case_type, case_action)?;
                }
                write!(f, "}}")
            }
            SStatementEnum::IndexFixed(st, i) => write!(f, "{st}.{i}"),
        }
    }
}
impl Display for VData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}
impl Display for VDataEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{v}"),
            Self::Tuple(v) | Self::List(_, v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    if i != 0 {
                        write!(f, " {v}")?;
                    } else {
                        write!(f, "{v}")?;
                    }
                }
                match self {
                    Self::List(..) => write!(f, "...")?,
                    _ => (),
                }
                write!(f, "]")?;
                Ok(())
            }
            Self::Function(v) => write!(f, "{v}"),
            Self::Thread(..) => write!(f, "THREAD"),
        }
    }
}
impl Display for RFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {:#?}", self.inputs.len(), self.block)
    }
}
