use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::{
    libs,
    script::{
        builtins,
        global_info::GlobalScriptInfo,
        val_data::VDataEnum,
        val_type::{VSingleType, VType},
    },
};

use super::{
    builtins::BuiltinFunction,
    code_macro::Macro,
    code_parsed::{SBlock, SFunction, SStatement, SStatementEnum},
    code_runnable::{RBlock, RFunction, RScript, RStatement, RStatementEnum},
};

pub enum ToRunnableError {
    MainWrongInput,
    UseOfUndefinedVariable(String),
    UseOfUndefinedFunction(String),
    CannotDeclareVariableWithDereference(String),
    CannotDereferenceTypeNTimes(VType, usize, VType),
    FunctionWrongArgCount(String, usize, usize),
    InvalidType {
        expected: VType,
        found: VType,
        problematic: VType,
    },
    CaseForceButTypeNotCovered(VType),
    MatchConditionInvalidReturn(VType),
    NotIndexableFixed(VType, usize),
    WrongInputsForBuiltinFunction(BuiltinFunction, String, Vec<VType>),
    WrongArgsForLibFunction(String, Vec<VType>),
    ForLoopContainerHasNoInnerTypes,
    StatementRequiresOutputTypeToBeAButItActuallyOutputsBWhichDoesNotFitInA(VType, VType, VType),
}
impl Debug for ToRunnableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
// TODO:
//  - Don't use {} to format, use .fmtgs(f, info) instead!
//  - Show location in code where the error was found
impl Display for ToRunnableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
                Self::MainWrongInput => write!(
                    f,
                    "Main function had the wrong input. This is a bug and should never happen."
                ),
                Self::UseOfUndefinedVariable(v) => write!(f, "Cannot use variable \"{v}\" as it isn't defined (yet?)."),
                Self::UseOfUndefinedFunction(v) => write!(f, "Cannot use function \"{v}\" as it isn't defined (yet?)."),
                Self::CannotDeclareVariableWithDereference(v) => write!(f, "Cannot declare a variable and dereference it (variable '{v}')."),
                Self::CannotDereferenceTypeNTimes(og_type, derefs_wanted, last_valid_type) => write!(f,
                    "Cannot dereference type {og_type} {derefs_wanted} times (stopped at {last_valid_type})."
                ),
                Self::FunctionWrongArgCount(v, a, b) => write!(f, "Tried to call function \"{v}\", which takes {a} arguments, with {b} arguments instead."),
                Self::InvalidType {
                    expected,
                    found,
                    problematic,
                } => {
                    write!(f, "Invalid type: Expected {expected} but found {found}, which includes {problematic}, which is not covered.")
                }
                Self::CaseForceButTypeNotCovered(v) => write!(f, "Switch! statement, but not all types covered. Types to cover: {v}"),
                Self::MatchConditionInvalidReturn(v) => write!(f, "match statement condition returned {v}, which is not necessarily a tuple of size 0 to 1."),
                Self::NotIndexableFixed(t, i) => write!(f, "Cannot use fixed-index {i} on type {t}."),
                Self::WrongInputsForBuiltinFunction(_builtin, builtin_name, args) => {
                    write!(f, "Wrong arguments for builtin function {}:", builtin_name)?;
                    for arg in args {
                        write!(f, " {arg}")?;
                    }
                    write!(f, ".")
                }
                Self::WrongArgsForLibFunction(name, args) => {
                    write!(f, "Wrong arguments for library function {}:", name)?;
                    for arg in args {
                        write!(f, " {arg}")?;
                    }
                    write!(f, ".")
                }
                Self::ForLoopContainerHasNoInnerTypes => {
                    write!(f, "For loop: container had no inner types, cannot iterate.")
                }
                Self::StatementRequiresOutputTypeToBeAButItActuallyOutputsBWhichDoesNotFitInA(required, real, problematic) => write!(f,
                    "the statement requires its output type to be {required}, but its real output type is {real}, which doesn not fit in the required type because of the problematic types {problematic}."
                ),
            }
    }
}

// Global, shared between all
pub struct GInfo {
    vars: usize,
    pub libs: Vec<libs::Lib>,
    pub lib_fns: HashMap<String, (usize, usize)>,
    pub enum_variants: HashMap<String, usize>,
}
impl Default for GInfo {
    fn default() -> Self {
        Self {
            vars: 0,
            libs: vec![],
            lib_fns: HashMap::new(),
            enum_variants: Self::default_enum_variants(),
        }
    }
}
impl GInfo {
    pub fn default_enum_variants() -> HashMap<String, usize> {
        builtins::EVS
            .iter()
            .enumerate()
            .map(|(i, v)| (v.to_string(), i))
            .collect()
    }
    pub fn new(libs: Vec<libs::Lib>, enum_variants: HashMap<String, usize>) -> Self {
        let mut lib_fns = HashMap::new();
        for (libid, lib) in libs.iter().enumerate() {
            for (fnid, (name, ..)) in lib.registered_fns.iter().enumerate() {
                lib_fns.insert(name.to_string(), (libid, fnid));
            }
        }
        Self {
            vars: 0,
            libs,
            lib_fns,
            enum_variants,
        }
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
    assert_eq!(
        s.inputs[0].1,
        VType {
            types: vec![VSingleType::List(VType {
                types: vec![VSingleType::String],
            })],
        }
    );
    let func = function(
        &s,
        &mut ginfo,
        LInfo {
            vars: HashMap::new(),
            fns: HashMap::new(),
        },
    )?;
    Ok(RScript::new(
        func,
        ginfo.vars,
        GlobalScriptInfo {
            libs: ginfo.libs,
            enums: ginfo.enum_variants,
        }
        .to_arc(),
    )?)
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
        }
        VSingleType::EnumVariantS(e, v) => {
            *t = VSingleType::EnumVariant(
                {
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
                },
            )
        }
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
                                problematic: VType { types: out },
                            });
                        }
                    }
                    RStatementEnum::FunctionCall(func.clone(), rargs)
                } else {
                    // TODO: type-checking for builtins
                    if let Some(builtin) = BuiltinFunction::get(v) {
                        let arg_types = rargs.iter().map(|v| v.out()).collect();
                        if builtin.can_take(&arg_types) {
                            RStatementEnum::BuiltinFunction(builtin, rargs)
                        } else {
                            return Err(ToRunnableError::WrongInputsForBuiltinFunction(builtin, v.to_string(), arg_types));
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
                                    ToRunnableError::WrongArgsForLibFunction(v.to_string(), rargs.iter().map(|v| v.out()).collect())
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
                            problematic: VType { types: out },
                        });
                    }
                },
                statement(&t, ginfo, linfo)?,
                match e {
                    Some(v) => Some(statement(&v, ginfo, linfo)?),
                    None => None,
                },
            ),
            SStatementEnum::Loop(c) => RStatementEnum::Loop(
                statement(&c, ginfo, linfo)?
            ),
            SStatementEnum::For(v, c, b) => {
                let mut linfo = linfo.clone();
                let container = statement(&c, ginfo, &mut linfo)?;
                let inner = container.out().inner_types();
                if inner.types.is_empty() {
                    return Err(ToRunnableError::ForLoopContainerHasNoInnerTypes);
                }
                linfo
                    .vars
                    .insert(v.clone(), (ginfo.vars, inner));
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
                                    fn make_readable(v: &mut VType, ginfo: &GInfo) {
                                        for t in v.types.iter_mut() {
                                            match t {
                                                VSingleType::EnumVariant(i, v) => {
                                                    let mut v = v.clone();
                                                    make_readable(&mut v, ginfo);
                                                    *t = VSingleType::EnumVariantS(ginfo.enum_variants.iter().find_map(|(st, us)| if *us == *i { Some(st.clone()) } else { None }).unwrap(), v);
                                                },
                                                VSingleType::EnumVariantS(_, v) => make_readable(v, ginfo),
                                                VSingleType::Tuple(v) => for t in v.iter_mut() {
                                                    make_readable(t, ginfo)
                                                }
                                                VSingleType::List(t) => make_readable(t, ginfo),
                                                VSingleType::Reference(v) => {
                                                    let mut v = v.clone().to();
                                                    make_readable(&mut v, ginfo);
                                                    assert_eq!(v.types.len(), 1);
                                                    *t = VSingleType::Reference(Box::new(v.types.remove(0)));
                                                }
                                                VSingleType::Bool | VSingleType::Int | VSingleType::Float | VSingleType::String | VSingleType::Function(..) | VSingleType::Thread(..) => (),
                                            }
                                        }
                                    }
                                    make_readable(&mut v, &ginfo);
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
            SStatementEnum::Macro(m) => match m {
                Macro::StaticMers(val) => RStatementEnum::Value(val.clone()),
            },
        }
        .to();
    // if force_output_type is set, verify that the real output type actually fits in the forced one.
    if let Some(force_opt) = &s.force_output_type {
        let real_output_type = statement.out();
        let problematic_types = real_output_type.fits_in(force_opt);
        if problematic_types.is_empty() {
            statement.force_output_type = Some(force_opt.clone());
        } else {
            return Err(ToRunnableError::StatementRequiresOutputTypeToBeAButItActuallyOutputsBWhichDoesNotFitInA(force_opt.clone(), real_output_type, VType { types: problematic_types }));
        }
    }
    if let Some((opt, derefs)) = &s.output_to {
        if let Some((var_id, var_out)) = linfo.vars.get(opt) {
            let out = statement.out();
            let mut var_derefd = var_out.clone();
            for _ in 0..*derefs {
                var_derefd = if let Some(v) = var_derefd.dereference() {
                    v
                } else {
                    return Err(ToRunnableError::CannotDereferenceTypeNTimes(
                        var_out.clone(),
                        *derefs,
                        var_derefd,
                    ));
                }
            }
            let inv_types = out.fits_in(&var_derefd);
            if !inv_types.is_empty() {
                eprintln!("Warn: shadowing variable {opt} because statement's output type {out} does not fit in the original variable's {var_out}. This might become an error in the future, or it might stop shadowing the variiable entirely - for stable scripts, avoid this by giving the variable a different name.");
                if *derefs != 0 {
                    return Err(ToRunnableError::CannotDeclareVariableWithDereference(
                        opt.clone(),
                    ));
                }
                linfo.vars.insert(opt.clone(), (ginfo.vars, out));
                statement.output_to = Some((ginfo.vars, 0));
                ginfo.vars += 1;
            } else {
                // mutate existing variable
                statement.output_to = Some((*var_id, *derefs));
            }
        } else {
            let mut out = statement.out();
            for _ in 0..*derefs {
                out = if let Some(v) = out.dereference() {
                    v
                } else {
                    return Err(ToRunnableError::CannotDereferenceTypeNTimes(
                        statement.out(),
                        *derefs,
                        out,
                    ));
                }
            }
            linfo.vars.insert(opt.clone(), (ginfo.vars, out));
            statement.output_to = Some((ginfo.vars, *derefs));
            ginfo.vars += 1;
        }
    }
    Ok(statement)
}
