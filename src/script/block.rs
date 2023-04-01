// A code block is any section of code. It contains its own local variables and functions, as well as a list of statements.
// Types starting with S are directly parsed from Strings and unchecked. Types starting with T are type-checked templates for R-types. Types starting with R are runnable. S are converted to T after parsing is done, and T are converted to R whenever they need to run.

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::libs;

use self::to_runnable::{ToRunnableError, GInfo};

use super::{
    builtins::BuiltinFunction,
    val_data::{VData, VDataEnum},
    val_type::{VSingleType, VType},
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
    Variable(String, bool),
    FunctionCall(String, Vec<SStatement>),
    FunctionDefinition(Option<String>, SFunction),
    Block(SBlock),
    If(SStatement, SStatement, Option<SStatement>),
    While(SStatement),
    For(String, SStatement, SStatement),
    Switch(String, Vec<(VType, SStatement)>, bool),
    Match(String, Vec<(SStatement, SStatement)>),
    IndexFixed(SStatement, usize),
    EnumVariant(String, SStatement),
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

pub fn to_runnable(f: SFunction, ginfo: GInfo) -> Result<RScript, ToRunnableError> {
    to_runnable::to_runnable(f, ginfo)
}

pub mod to_runnable {
    use std::{
        collections::HashMap,
        fmt::{Debug, Display},
        sync::Arc,
    };

    use crate::{script::{
        val_data::VDataEnum,
        val_type::{VSingleType, VType},
    }, libs};

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
        MatchConditionInvalidReturn(VType),
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
                Self::MatchConditionInvalidReturn(v) => write!(f, "match statement condition returned {v}, which is not necessarily a tuple of size 0 to 1."),
                Self::NotIndexableFixed(t, i) => write!(f, "Cannot use fixed-index {i} on type {t}."),
            }
        }
    }

    // Global, shared between all
    pub struct GInfo {
        vars: usize,
        libs: Arc<Vec<libs::Lib>>,
        lib_fns: HashMap<String, (usize, usize)>,
        enum_variants: HashMap<String, usize>,
    }
    impl GInfo {
        pub fn new(libs: Arc<Vec<libs::Lib>>) -> Self {
            let mut lib_fns = HashMap::new();
            for (libid, lib) in libs.iter().enumerate() {
                for (fnid, (name, ..)) in lib.registered_fns.iter().enumerate() {
                    lib_fns.insert(name.to_string(), (libid, fnid));
                }
            }
            Self { vars: 0, libs, lib_fns, enum_variants: HashMap::new() }
        }
    }
    // Local, used to keep local variables separated
    #[derive(Clone)]
    struct LInfo {
        vars: HashMap<String, (usize, VType)>,
        fns: HashMap<String, Arc<RFunction>>,
    }

    pub fn to_runnable(s: SFunction, mut ginfo: GInfo) -> Result<RScript, ToRunnableError> {
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
        let func = function(
            &s,
            &mut ginfo,
            LInfo {
                vars: HashMap::new(),
                fns: HashMap::new(),
            },
        )?;
        Ok(RScript::new(func, ginfo.vars, ginfo.libs)?)
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

    fn stypes(t: &mut VType, ginfo: &mut GInfo) {
        for t in &mut t.types {
            stype(t, ginfo);
        }
    }
    fn stype(t: &mut VSingleType, ginfo: &mut GInfo) {
        match t {
            VSingleType::Tuple(v) => {
                for t in v {
                    stypes(t, ginfo);
                }
            },
            VSingleType::EnumVariantS(e, v) => *t = VSingleType::EnumVariant({
                if let Some(v) = ginfo.enum_variants.get(e) {
                    *v
                } else {
                    let v = ginfo.enum_variants.len();
                    ginfo.enum_variants.insert(e.clone(), v);
                    v
                }
            },
            {
                stypes(v, ginfo);
                v.clone()
            }
            ),
            _ => (),
        }
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
            SStatementEnum::Variable(v, is_ref) => {
                if let Some(var) = linfo.vars.get(v) {
                    RStatementEnum::Variable(var.0, {
                        let mut v = var.1.clone(); stypes(&mut v, ginfo); v }, *is_ref)
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
                        if builtin.can_take(&rargs.iter().map(|v| v.out()).collect()) {
                            RStatementEnum::BuiltinFunction(builtin, rargs)
                        } else {
                            todo!("ERR: Builtin function \"{v}\" with wrong args - this isn't a proper error yet, sorry.");
                        }
                    } else {
                        // LIBRARY FUNCTION?
                        if let Some((libid, fnid)) = ginfo.lib_fns.get(v) {
                            let (_name, fn_in, fn_out) = &ginfo.libs[*libid].registered_fns[*fnid];
                            if fn_in.len() == rargs.len() && fn_in.iter().zip(rargs.iter()).all(|(fn_in, arg)| arg.out().fits_in(fn_in).is_empty()) {
                                RStatementEnum::LibFunction(*libid, *fnid, rargs, fn_out.clone())
                            } else {
                                // TODO! better error here
                                return Err(if fn_in.len() == rargs.len() {
                                    todo!("Err: Wrong args for LibFunction \"{v}\".");
                                } else {
                                    ToRunnableError::FunctionWrongArgCount(v.to_string(), fn_in.len(), rargs.len())
                                });
                            }
                        } else {
                        return Err(ToRunnableError::UseOfUndefinedFunction(v.clone()));
                        }
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
                    let og_type = switch_on_v.1.clone(); // linfo.vars.get(switch_on).unwrap().1.clone();
                    for case in cases {
                        let case0 = { let mut v = case.0.clone(); stypes(&mut v, ginfo); v };
                        linfo.vars.get_mut(switch_on).unwrap().1 = case0.clone();
                        ncases.push((case0, statement(&case.1, ginfo, linfo)?));
                    }
                    linfo.vars.get_mut(switch_on).unwrap().1 = og_type;

                    let switch_on_out = switch_on_v.1;
                    if *force {
                        let mut types_not_covered_req_error = false;
                        let mut types_not_covered = VType { types: vec![] };
                        for val_type in switch_on_out.types.iter() {
                            let val_type: VType = val_type.clone().into();
                            let mut linf2 = linfo.clone();
                            linf2.vars.get_mut(switch_on).unwrap().1 = val_type.clone();
                            'force: {
                                for (case_type, _) in cases {
                                    let mut ct = case_type.clone();
                                    stypes(&mut ct, ginfo);
                                    if val_type.fits_in(&ct).is_empty() {
                                        break 'force;
                                    }
                                }
                                types_not_covered_req_error = true;
                                types_not_covered = types_not_covered | {
                                    let mut v = val_type;
                                    for t in v.types.iter_mut() {
                                        if let VSingleType::EnumVariant(i, v) = t {
                                            *t = VSingleType::EnumVariantS(ginfo.enum_variants.iter().find_map(|(st, us)| if *us == *i { Some(st.clone()) } else { None }).unwrap(), v.clone());
                                        }
                                    }
                                    v
                                };
                            }
                        }
                        if types_not_covered_req_error {
                            return Err(ToRunnableError::CaseForceButTypeNotCovered(types_not_covered));
                        }
                    }
                    RStatementEnum::Switch(
                        RStatementEnum::Variable(switch_on_v.0, switch_on_out, false).to(),
                        ncases,
                    )
                } else {
                    return Err(ToRunnableError::UseOfUndefinedVariable(switch_on.clone()));
                }
            }
            SStatementEnum::Match(match_on, cases) => {
                if let Some(switch_on_v) = linfo.vars.get(match_on).cloned() {
                    let mut ncases = Vec::with_capacity(cases.len());
                    let og_type = switch_on_v.1.clone(); // linfo.vars.get(match_on).unwrap().1.clone();
                    for case in cases {
                        let case_condition = statement(&case.0, ginfo, linfo)?;
                        let case_condition_out =  case_condition.out();
                        let mut refutable = false;
                        let mut success_output = VType { types: vec![] };
                        for case_type in case_condition_out.types.iter() {
                            match case_type {
                            VSingleType::Tuple(tuple) =>
                                match tuple.len() {
                                    0 => refutable = true,
                                    1 => success_output = success_output | &tuple[0],
                                    _ => return Err(ToRunnableError::MatchConditionInvalidReturn(case_condition_out)),
                                },
                                VSingleType::Bool => {
                                    refutable = true;
                                    success_output = success_output | VSingleType::Bool.to()
                                }
                                _ => success_output = success_output | case_type.clone().to(),
                            }
                        }
                        if refutable == false {
                            eprintln!("WARN: Irrefutable match condition with return type {}", case_condition_out);
                        }
                        if !success_output.types.is_empty() {
                            let var = linfo.vars.get_mut(match_on).unwrap();
                            let og = var.1.clone();
                            var.1 = success_output;
                            let case_action = statement(&case.1, ginfo, linfo)?;
                            linfo.vars.get_mut(match_on).unwrap().1 = og;
                            ncases.push((case_condition, case_action));
                        } else {
                            eprintln!("WARN: Match condition with return type {} never returns a match and will be ignored entirely. Note: this also skips type-checking for the action part of this match arm because the success type is not known.", case_condition_out);
                        }
                        
                    }
                    linfo.vars.get_mut(match_on).unwrap().1 = og_type;

                    RStatementEnum::Match(
                        switch_on_v.0,
                        ncases,
                    )
                } else {
                    return Err(ToRunnableError::UseOfUndefinedVariable(match_on.clone()));
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
            SStatementEnum::EnumVariant(variant, s) => RStatementEnum::EnumVariant({
                if let Some(v) = ginfo.enum_variants.get(variant) {
                    *v
                } else {
                    let v =  ginfo.enum_variants.len();
                    ginfo.enum_variants.insert(variant.clone(), v);
                    v
                }
            }, statement(s, ginfo, linfo)?),
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
    pub fn run(&self, vars: &Vec<Am<VData>>, libs: &Arc<Vec<libs::Lib>>) -> VData {
        let mut last = None;
        for statement in &self.statements {
            last = Some(statement.run(vars, libs));
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
    pub fn run(&self, vars: &Vec<Am<VData>>, libs: &Arc<Vec<libs::Lib>>) -> VData {
        self.block.run(vars, libs)
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
    pub fn out_vt(&self, input_types: &Vec<VType>) -> VType {
        let mut out = VType { types: vec![] };
        for (itype, otype) in self.input_output_map.iter() {
            if itype
                .iter()
                .zip(input_types.iter())
                .all(|(expected, got)| got.contains(expected))
            {
                out = out | otype;
            }
        }
        out
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
    pub fn run(&self, vars: &Vec<Am<VData>>, libs: &Arc<Vec<libs::Lib>>) -> VData {
        let out = self.statement.run(vars, libs);
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
    Variable(usize, VType, bool), // Arc<Mutex<..>> here, because imagine variable in for loop that is used in a different thread -> we need multiple "same" variables
    FunctionCall(Arc<RFunction>, Vec<RStatement>),
    BuiltinFunction(BuiltinFunction, Vec<RStatement>),
    LibFunction(usize, usize, Vec<RStatement>, VType),
    Block(RBlock),
    If(RStatement, RStatement, Option<RStatement>),
    While(RStatement),
    For(usize, RStatement, RStatement),
    Switch(RStatement, Vec<(VType, RStatement)>),
    Match(usize, Vec<(RStatement, RStatement)>),
    IndexFixed(RStatement, usize),
    EnumVariant(usize, RStatement)
}
impl RStatementEnum {
    pub fn run(&self, vars: &Vec<Am<VData>>, libs: &Arc<Vec<libs::Lib>>) -> VData {
        match self {
            Self::Value(v) => v.clone(),
            Self::Tuple(v) => {
                let mut w = vec![];
                for v in v {
                    w.push(v.run(vars, libs));
                }
                VDataEnum::Tuple(w).to()
            }
            Self::List(v) => {
                let mut w = vec![];
                let mut out = VType { types: vec![] };
                for v in v {
                    let val = v.run(vars, libs);
                    out = out | val.out();
                    w.push(val);
                }
                VDataEnum::List(out, w).to()
            }
            Self::Variable(v, _, is_ref) => {
                if *is_ref {
                    VDataEnum::Reference(vars[*v].clone()).to()
                } else {
                    vars[*v].lock().unwrap().clone()
                }
            }
            Self::FunctionCall(func, args) => {
                for (i, input) in func.inputs.iter().enumerate() {
                    *vars[*input].lock().unwrap() = args[i].run(vars, libs);
                }
                func.run(vars, libs)
            }
            Self::BuiltinFunction(v, args) => v.run(args, vars, libs),
            Self::LibFunction(libid, fnid, args, _) => libs[*libid].run_fn(*fnid, &args.iter().map(|arg| arg.run(vars, libs)).collect()),
            Self::Block(b) => b.run(vars, libs),
            Self::If(c, t, e) => {
                if let VDataEnum::Bool(v) = c.run(vars, libs).data {
                    if v {
                        t.run(vars, libs)
                    } else {
                        if let Some(e) = e {
                            e.run(vars, libs)
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            Self::While(c) => loop {
                // While loops will break if the value matches.
                if let Some(break_val) = c.run(vars, libs).data.matches() {
                    break break_val;
                }
            },
            Self::For(v, c, b) => {
                // matching values also break with value from a for loop.
                let c = c.run(vars, libs);
                let mut vars = vars.clone();
                let mut in_loop = |c| {
                    vars[*v] = Arc::new(Mutex::new(c));
                    b.run(&vars, libs)
                };

                let mut oval = VDataEnum::Tuple(vec![]).to();
                match c.data {
                    VDataEnum::Int(v) => {
                        for i in 0..v {
                            if let Some(v) = in_loop(VDataEnum::Int(i).to()).data.matches() {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::String(v) => {
                        for ch in v.chars() {
                            if let Some(v) = in_loop(VDataEnum::String(ch.to_string()).to()).data.matches() {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                        for v in v {
                            if let Some(v) = in_loop(v).data.matches() {
                                oval = v;
                                break;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                oval
            }
            Self::Switch(switch_on, cases) => {
                let switch_on = switch_on.run(vars, libs);
                let switch_on_type = switch_on.out();
                let mut out = VDataEnum::Tuple(vec![]).to();
                for (case_type, case_action) in cases.iter() {
                    if switch_on_type.fits_in(case_type).is_empty() {
                        out = case_action.run(vars, libs);
                        break;
                    }
                }
                out
            }
            Self::Match(match_on, cases) => 'm: {
                for (case_condition, case_action) in cases {
                    // [t] => Some(t), t => Some(t), [] | false => None
                    if let Some(v) = case_condition.run(vars, libs).data.matches() {
                        let og = {
                            std::mem::replace(&mut *vars[*match_on].lock().unwrap(), v)
                        };
                        let res = case_action.run(vars, libs);
                        *vars[*match_on].lock().unwrap() = og;
                        break 'm res;
                    }
                }
                VDataEnum::Tuple(vec![]).to()
            }
            Self::IndexFixed(st, i) => st.run(vars, libs).get(*i).unwrap(),
            Self::EnumVariant(e, v) => {
                VDataEnum::EnumVariant(*e, Box::new(v.run(vars, libs))).to()
            }
        }
    }
    pub fn out(&self) -> VType {
        match self {
            Self::Value(v) => v.out(),
            Self::Tuple(v) => VSingleType::Tuple(v.iter().map(|v| v.out()).collect()).into(),
            Self::List(v) => VSingleType::List({
                let mut types = VType { types: vec![] };
                for t in v {
                    types = types | t.out();
                }
                types
            })
            .into(),
            Self::Variable(_, t, is_ref) => {
                if *is_ref {
                    VType {
                        types: t
                            .types
                            .iter()
                            .map(|t| VSingleType::Reference(Box::new(t.clone())))
                            .collect(),
                    }
                } else {
                    t.clone()
                }
            }
            Self::FunctionCall(f, args) => f.out_vt(&args.iter().map(|v| v.out()).collect()),
            Self::LibFunction(.., out) => out.clone(),
            Self::Block(b) => b.out(),
            Self::If(_, a, b) => {
                if let Some(b) = b {
                    a.out() | b.out()
                } else {
                    a.out()
                }
            }
            Self::While(c) => {
                c.out().matches().1
            }
            Self::For(_, _, b) => {
                VSingleType::Tuple(vec![]).to() | b.out().matches().1
            }
            Self::BuiltinFunction(f, args) => f.returns(args.iter().map(|rs| rs.out()).collect()),
            Self::Switch(switch_on, cases) => {
                let switch_on = switch_on.out().types;
                let mut might_return_empty = switch_on.is_empty();
                let mut out = VType { types: vec![] }; // if nothing is executed
                for switch_on in switch_on {
                    let switch_on = switch_on.to();
                    'search: {
                        for (on_type, case) in cases.iter() {
                            if switch_on.fits_in(&on_type).is_empty() {
                                out = out | case.out();
                                break 'search;
                            }
                        }
                        might_return_empty = true;
                    }
                }
                if might_return_empty {
                    out = out | VSingleType::Tuple(vec![]).to();
                }
                out
            }
            Self::Match(_, cases) => {
                let mut out = VSingleType::Tuple(vec![]).to();
                for case in cases {
                    out = out | case.1.out();
                }
                out
            }
            Self::IndexFixed(st, i) => st.out().get(*i).unwrap(),
            Self::EnumVariant(e, v) => VSingleType::EnumVariant(*e, v.out()).to(),
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
    libs: Arc<Vec<libs::Lib>>,
}
impl RScript {
    fn new(main: RFunction, vars: usize, libs: Arc<Vec<libs::Lib>>) -> Result<Self, ToRunnableError> {
        if main.inputs.len() != 1 {
            return Err(ToRunnableError::MainWrongInput);
        }
        Ok(Self { main, vars, libs })
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
        self.main.run(&vars, &self.libs)
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
            Self::Function(_) => write!(f, "FUNCTION"),
            Self::Thread(_) => write!(f, "THREAD"),
            Self::Reference(r) => write!(f, "&{r}"),
            Self::EnumVariant(v, t) => write!(f, "{v}: {t}"),
            Self::EnumVariantS(v, t) => write!(f, "{v}: {t}"),
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
            SStatementEnum::Variable(v, is_ref) => {
                write!(f, "{}{v}", if *is_ref { "&" } else { "" })
            }
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
            SStatementEnum::Match(match_on, cases) => {
                writeln!(f, "match {match_on} {{")?;
                for (case_cond, case_action) in cases.iter() {
                    writeln!(f, "{} {}", case_cond, case_action)?;
                }
                write!(f, "}}")
            }
            SStatementEnum::IndexFixed(st, i) => write!(f, "{st}.{i}"),
            SStatementEnum::EnumVariant(e, s) => write!(f, "{e}: {s}"),
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
            Self::Reference(r) => write!(f, "{}", r.lock().unwrap()),
            Self::EnumVariant(v, d) => write!(f, "{v}: {d}"),
        }
    }
}
impl Display for RFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {:#?}", self.inputs.len(), self.block)
    }
}
