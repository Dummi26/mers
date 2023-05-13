use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{
    lang::{
        builtins,
        global_info::GlobalScriptInfo,
        val_data::{VData, VDataEnum},
        val_type::{VSingleType, VType},
    },
    libs,
};

use super::{
    builtins::BuiltinFunction,
    code_macro::Macro,
    code_parsed::{SBlock, SFunction, SStatement, SStatementEnum},
    code_runnable::{RBlock, RFunction, RScript, RStatement, RStatementEnum},
    global_info::GSInfo,
};

pub enum ToRunnableError {
    MainWrongInput,
    UseOfUndefinedVariable(String),
    UseOfUndefinedFunction(String),
    UnknownType(String),
    CannotDeclareVariableWithDereference(String),
    CannotDereferenceTypeNTimes(VType, usize, VType),
    FunctionWrongArgCount(String, usize, usize),
    FunctionWrongArgs(Vec<VType>, String),
    InvalidType {
        expected: VType,
        found: VType,
        problematic: VType,
    },
    CannotAssignTo(VType, VType),
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
        self.fmtgs(f, None)
    }
}
impl ToRunnableError {
    pub fn fmtgs(
        &self,
        f: &mut std::fmt::Formatter,
        info: Option<&GlobalScriptInfo>,
    ) -> std::fmt::Result {
        match self {
                Self::MainWrongInput => write!(
                    f,
                    "Main function had the wrong input. This is a bug and should never happen."
                ),
                Self::UseOfUndefinedVariable(v) => write!(f, "Cannot use variable \"{v}\" as it isn't defined (yet?)."),
                Self::UseOfUndefinedFunction(v) => write!(f, "Cannot use function \"{v}\" as it isn't defined (yet?)."),
                Self::UnknownType(name) => write!(f, "Unknown type \"{name}\"."),
                Self::CannotDeclareVariableWithDereference(v) => write!(f, "Cannot declare a variable and dereference it (variable '{v}')."),
                Self::CannotDereferenceTypeNTimes(og_type, derefs_wanted, last_valid_type) => {
                    write!(f, "Cannot dereference type ")?;
                    og_type.fmtgs(f, info)?;
                    write!(f, " {derefs_wanted} times (stopped at ")?;
                    last_valid_type.fmtgs(f, info);
                    write!(f, ")")?;
                    Ok(())
                },
                Self::FunctionWrongArgCount(v, a, b) => write!(f, "Tried to call function \"{v}\", which takes {a} arguments, with {b} arguments instead."),
                Self::FunctionWrongArgs(args, name) => write!(f, "Wrong args for function \"{name}\":{}", args.iter().map(|v| format!(" {v}")).collect::<String>()),
                Self::InvalidType {
                    expected,
                    found,
                    problematic,
                } => {
                    write!(f, "Invalid type: Expected ")?;
                    expected.fmtgs(f, info)?;
                    write!(f, " but found ")?;
                    found.fmtgs(f, info)?;
                    write!(f, ", which includes ")?;
                    problematic.fmtgs(f, info)?;
                    write!(f, " which is not covered.")?;
                    Ok(())
                }
                Self::CaseForceButTypeNotCovered(v) => {
                    write!(f, "Switch! statement, but not all types covered. Types to cover: ")?;
                    v.fmtgs(f, info)?;
                    Ok(())
                }
                Self::MatchConditionInvalidReturn(v) => {
                    write!(f, "match statement condition returned ")?;
                    v.fmtgs(f, info)?;
                    write!(f, ", which is not necessarily a tuple of size 0 to 1.")?;
                    Ok(())
                }
                Self::NotIndexableFixed(t, i) => {
                    write!(f, "Cannot use fixed-index {i} on type ")?;
                    t.fmtgs(f, info)?;
                    write!(f, ".")?;
                    Ok(())
                }
                Self::WrongInputsForBuiltinFunction(_builtin, builtin_name, args) => {
                    write!(f, "Wrong arguments for builtin function \"{}\":", builtin_name)?;
                    for arg in args {
                        write!(f, " ")?;
                        arg.fmtgs(f, info)?;
                    }
                    write!(f, ".")
                }
                Self::WrongArgsForLibFunction(name, args) => {
                    write!(f, "Wrong arguments for library function {}:", name)?;
                    for arg in args {
                        write!(f, " ")?;
                        arg.fmtgs(f, info)?;
                    }
                    write!(f, ".")
                }
                Self::CannotAssignTo(val, target) => {
                    write!(f, "Cannot assign type ")?;
                    val.fmtgs(f, info)?;
                    write!(f, " to ")?;
                    target.fmtgs(f, info)?;
                    write!(f, ".")?;
                    Ok(())
                },
                Self::ForLoopContainerHasNoInnerTypes => {
                    write!(f, "For loop: container had no inner types, cannot iterate.")
                }
                Self::StatementRequiresOutputTypeToBeAButItActuallyOutputsBWhichDoesNotFitInA(required, real, problematic) => {
                    write!(f, "the statement requires its output type to be ")?;
                    required.fmtgs(f, info)?;
                    write!(f, ", but its real output type is ")?;
                    real.fmtgs(f, info)?;
                    write!(f, ", which doesn't fit in the required type because of the problematic types ")?;
                    problematic.fmtgs(f, info)?;
                    write!(f, ".")?;
                    Ok(())
                }
            }
    }
}

// Local, used to keep local variables separated
#[derive(Clone)]
struct LInfo {
    vars: HashMap<String, (Arc<Mutex<VData>>, VType)>,
    fns: HashMap<String, Arc<RFunction>>,
}

pub fn to_runnable(
    s: SFunction,
    mut ginfo: GlobalScriptInfo,
) -> Result<RScript, (ToRunnableError, GSInfo)> {
    if s.inputs.len() != 1 || s.inputs[0].0 != "args" {
        return Err((ToRunnableError::MainWrongInput, ginfo.to_arc()));
    }
    assert_eq!(
        s.inputs[0].1,
        VType {
            types: vec![VSingleType::List(VType {
                types: vec![VSingleType::String],
            })],
        }
    );
    let func = match function(
        &s,
        &mut ginfo,
        LInfo {
            vars: HashMap::new(),
            fns: HashMap::new(),
        },
    ) {
        Ok(v) => v,
        Err(e) => return Err((e, ginfo.to_arc())),
    };
    let ginfo = ginfo.to_arc();
    match RScript::new(func, ginfo.clone()) {
        Ok(v) => Ok(v),
        Err(e) => Err((e, ginfo)),
    }
}

// go over every possible known-type input for the given function, returning all possible RFunctions.
fn get_all_functions(
    s: &SFunction,
    ginfo: &mut GlobalScriptInfo,
    linfo: &mut LInfo,
    input_vars: &Vec<Arc<Mutex<VData>>>,
    inputs: &mut Vec<VSingleType>,
    out: &mut Vec<(Vec<VSingleType>, VType)>,
) -> Result<(), ToRunnableError> {
    if s.inputs.len() > inputs.len() {
        let input_here = &s.inputs[inputs.len()].1;
        for t in &input_here.types {
            let mut t = t.clone();
            stype(&mut t, ginfo)?;
            inputs.push(t);
            get_all_functions(s, ginfo, linfo, input_vars, inputs, out)?;
            inputs.pop();
        }
        Ok(())
    } else {
        // set the types
        for (varid, vartype) in s.inputs.iter().zip(inputs.iter()) {
            linfo.vars.get_mut(&varid.0).unwrap().1 = {
                let mut vartype = vartype.clone();
                stype(&mut vartype, ginfo)?;
                vartype.to()
            }
        }
        out.push((
            inputs.clone(),
            block(&s.block, ginfo, linfo.clone())?.out(ginfo),
        ));
        Ok(())
    }
}
fn function(
    s: &SFunction,
    ginfo: &mut GlobalScriptInfo,
    mut linfo: LInfo,
) -> Result<RFunction, ToRunnableError> {
    let mut input_vars = vec![];
    let mut input_types = vec![];
    for (iname, itype) in &s.inputs {
        let mut itype = itype.to_owned();
        stypes(&mut itype, ginfo)?;
        let var = Arc::new(Mutex::new(VData::new_placeholder()));
        linfo
            .vars
            .insert(iname.clone(), (Arc::clone(&var), itype.clone()));
        input_vars.push(var);
        input_types.push(itype);
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

fn block(
    s: &SBlock,
    ginfo: &mut GlobalScriptInfo,
    mut linfo: LInfo,
) -> Result<RBlock, ToRunnableError> {
    let mut statements = Vec::new();
    for st in &s.statements {
        statements.push(statement(st, ginfo, &mut linfo)?);
    }
    Ok(RBlock { statements })
}

pub fn stypes(t: &mut VType, ginfo: &mut GlobalScriptInfo) -> Result<(), ToRunnableError> {
    for t in &mut t.types {
        stype(t, ginfo)?;
    }
    Ok(())
}
pub fn stype(t: &mut VSingleType, ginfo: &mut GlobalScriptInfo) -> Result<(), ToRunnableError> {
    match t {
        VSingleType::Bool | VSingleType::Int | VSingleType::Float | VSingleType::String => (),
        VSingleType::Tuple(v) => {
            for t in v {
                stypes(t, ginfo)?;
            }
        }
        VSingleType::List(t) => stypes(t, ginfo)?,
        VSingleType::Reference(t) => stype(t, ginfo)?,
        VSingleType::Thread(t) => stypes(t, ginfo)?,
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
                    stypes(v, ginfo)?;
                    v.clone()
                },
            )
        }
        VSingleType::Function(io_map) => {
            for io_variant in io_map {
                for i in &mut io_variant.0 {
                    stype(i, ginfo)?;
                }
                stypes(&mut io_variant.1, ginfo)?;
            }
        }
        VSingleType::EnumVariant(_, t) => stypes(t, ginfo)?,
        VSingleType::CustomTypeS(name) => {
            *t = VSingleType::CustomType(
                if let Some(v) = ginfo.custom_type_names.get(&name.to_lowercase()) {
                    *v
                } else {
                    return Err(ToRunnableError::UnknownType(name.to_owned()));
                },
            )
        }
        VSingleType::CustomType(_) => (),
    }
    Ok(())
}
fn statement(
    s: &SStatement,
    ginfo: &mut GlobalScriptInfo,
    linfo: &mut LInfo,
) -> Result<RStatement, ToRunnableError> {
    statement_adv(s, ginfo, linfo, None)
}
fn statement_adv(
    s: &SStatement,
    ginfo: &mut GlobalScriptInfo,
    linfo: &mut LInfo,
    // if Some((t, is_init)), the statement creates by this function is the left side of an assignment, meaning it can create variables. t is the type that will be assigned to it.
    to_be_assigned_to: Option<(VType, &mut bool)>,
) -> Result<RStatement, ToRunnableError> {
    let mut state = match &*s.statement {
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
                if !linfo.vars.contains_key(v) {
                    if let Some((t, is_init)) = to_be_assigned_to {
                        *is_init = true;
                        linfo.vars.insert(v.to_owned(), (Arc::new(Mutex::new(VData::new_placeholder())), t));
                    }
                }
                if let Some(var) = linfo.vars.get(v) {
                    RStatementEnum::Variable(
                        Arc::clone(&var.0),
                        {
                            let mut v = var.1.clone();
                            stypes(&mut v, ginfo)?;
                            v
                        },
                        *is_ref
                    )
                } else {
                        return Err(ToRunnableError::UseOfUndefinedVariable(v.clone()));
                }
            }
            SStatementEnum::FunctionCall(v, args) => {
                let mut rargs = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    rargs.push(statement(arg, ginfo, linfo)?);
                }
                fn check_fn_args(args: &Vec<VType>, inputs: &Vec<(Vec<VType>, VType)>, ginfo: &GlobalScriptInfo) -> Option<VType> {
                    let mut fit_any = false;
                    let mut out = VType::empty();
                    for (inputs, output) in inputs {
                        if args.len() == inputs.len() && args.iter().zip(inputs.iter()).all(|(arg, input)| arg.fits_in(input, ginfo).is_empty()) {
                            fit_any = true;
                            out = out | output;
                        }
                    }
                    if fit_any {
                        Some(out)
                    } else {
                        None
                    }
                }
                let arg_types: Vec<_> = rargs.iter().map(|v| v.out(ginfo)).collect();
                if let Some(func) = linfo.fns.get(v) {
                    if let Some(_out) = check_fn_args(&arg_types, &func.input_output_map.iter().map(|v| (v.0.iter().map(|v| v.clone().to()).collect(), v.1.to_owned())).collect(), ginfo) {
                        RStatementEnum::FunctionCall(func.clone(), rargs)
                    } else {
                        return Err(ToRunnableError::FunctionWrongArgs(
                            arg_types,
                            v.to_owned()
                        ));
                    }
                } else {
                    if let Some(builtin) = BuiltinFunction::get(v) {
                        let arg_types = rargs.iter().map(|v| v.out(ginfo)).collect();
                        if builtin.can_take(&arg_types, ginfo) {
                            RStatementEnum::BuiltinFunction(builtin, rargs)
                        } else {
                            return Err(ToRunnableError::WrongInputsForBuiltinFunction(builtin, v.to_string(), arg_types));
                        }
                    } else {
                        // LIBRARY FUNCTION?
                        if let Some((libid, fnid)) = ginfo.lib_fns.get(v) {
                            let lib = &ginfo.libs[*libid];
                            let libfn = &lib.registered_fns[*fnid];
                            if let Some(fn_out) = check_fn_args(&arg_types, &libfn.1, ginfo) {
                                RStatementEnum::LibFunction(*libid, *fnid, rargs, fn_out.clone())
                            } else {
                                return Err(ToRunnableError::WrongArgsForLibFunction(v.to_owned(), arg_types));
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
                        VDataEnum::Function(Arc::new(function(f, ginfo, linfo.clone())?)).to(),
                    )
                }
            }
            SStatementEnum::Block(b) => RStatementEnum::Block(block(&b, ginfo, linfo.clone())?),
            SStatementEnum::If(c, t, e) => RStatementEnum::If(
                {
                    let condition = statement(&c, ginfo, linfo)?;
                    let out = condition.out(ginfo).fits_in(&VType {
                        types: vec![VSingleType::Bool],
                    }, ginfo);
                    if out.is_empty() {
                        condition
                    } else {
                        return Err(ToRunnableError::InvalidType {
                            expected: VSingleType::Bool.into(),
                            found: condition.out(ginfo),
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
                let inner = container.out(ginfo).inner_types();
                if inner.types.is_empty() {
                    return Err(ToRunnableError::ForLoopContainerHasNoInnerTypes);
                }
                let for_loop_var = Arc::new(Mutex::new(VData::new_placeholder()));
                linfo
                    .vars
                    .insert(v.clone(), (Arc::clone(&for_loop_var), inner));
                let block = statement(&b, ginfo, &mut linfo)?;
                let o = RStatementEnum::For(for_loop_var, container, block);
                o
            }

            SStatementEnum::Switch(switch_on, cases, force) => {
                if let Some(switch_on_v) = linfo.vars.get(switch_on).cloned() {
                    let mut ncases = Vec::with_capacity(cases.len());
                    let og_type = switch_on_v.1.clone(); // linfo.vars.get(switch_on).unwrap().1.clone();
                    for case in cases {
                        let case0 = { let mut v = case.0.clone(); stypes(&mut v, ginfo)?; v };
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
                                    stypes(&mut ct, ginfo)?;
                                    if val_type.fits_in(&ct, ginfo).is_empty() {
                                        break 'force;
                                    }
                                }
                                types_not_covered_req_error = true;
                                types_not_covered = types_not_covered | {
                                    let mut v = val_type;
                                    /// converts the VType to one that is human-readable (changes enum from usize to String, ...)
                                    fn make_readable(v: &mut VType, ginfo: &GlobalScriptInfo) {
                                        for t in v.types.iter_mut() {
                                            match t {
                                                VSingleType::EnumVariant(i, v) => {
                                                    let mut v = v.clone();
                                                    make_readable(&mut v, ginfo);
                                                    *t = VSingleType::EnumVariantS(ginfo.enum_variants.iter().find_map(|(st, us)| if *us == *i { Some(st.clone()) } else { None }).unwrap(), v);
                                                },
                                                VSingleType::CustomType(i) => {
                                                    *t = VSingleType::CustomTypeS(ginfo.custom_type_names.iter().find_map(|(st, us)| if *us == *i { Some(st.clone()) } else { None }).unwrap());
                                                }
                                                VSingleType::Tuple(v) => for t in v.iter_mut() {
                                                    make_readable(t, ginfo)
                                                }
                                                VSingleType::List(t) | VSingleType::EnumVariantS(_, t) => make_readable(t, ginfo),
                                                VSingleType::Reference(v) => {
                                                    let mut v = v.clone().to();
                                                    make_readable(&mut v, ginfo);
                                                    assert_eq!(v.types.len(), 1);
                                                    *t = VSingleType::Reference(Box::new(v.types.remove(0)));
                                                }
                                                VSingleType::Bool | VSingleType::Int | VSingleType::Float | VSingleType::String | VSingleType::Function(..) | VSingleType::Thread(..) | VSingleType::CustomTypeS(_) => (),
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
                        let case_condition_out =  case_condition.out(ginfo);
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
                if st.out(ginfo).get_always(*i, ginfo).is_some() {
                    RStatementEnum::IndexFixed(st, *i)
                } else {
                    return Err(ToRunnableError::NotIndexableFixed(st.out(ginfo), *i));
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
            SStatementEnum::TypeDefinition(name, t) => {
                // insert to name map has to happen before stypes()
                ginfo.custom_type_names.insert(name.to_lowercase(), ginfo.custom_types.len());
                let mut t = t.to_owned();
                stypes(&mut t, ginfo)?;
                ginfo.custom_types.push(t);
                RStatementEnum::Value(VDataEnum::Tuple(vec![]).to())
            }
            SStatementEnum::Macro(m) => match m {
                Macro::StaticMers(val) => RStatementEnum::Value(val.clone()),
            },
        }
        .to();
    // if force_output_type is set, verify that the real output type actually fits in the forced one.
    if let Some(force_opt) = &s.force_output_type {
        let mut force_opt = force_opt.to_owned();
        stypes(&mut force_opt, ginfo)?;
        let real_output_type = state.out(ginfo);
        let problematic_types = real_output_type.fits_in(&force_opt, ginfo);
        if problematic_types.is_empty() {
            state.force_output_type = Some(force_opt);
        } else {
            return Err(ToRunnableError::StatementRequiresOutputTypeToBeAButItActuallyOutputsBWhichDoesNotFitInA(force_opt.clone(), real_output_type, VType { types: problematic_types }));
        }
    }
    if let Some((opt, derefs)) = &s.output_to {
        let mut is_init = false;
        let optr = statement_adv(
            opt,
            ginfo,
            linfo,
            if *derefs == 0 {
                Some((state.out(ginfo), &mut is_init))
            } else {
                None
            },
        )?;
        let mut opt_type = optr.out(ginfo);
        for _ in 0..*derefs {
            if let Some(deref_type) = optr.out(ginfo).dereference() {
                opt_type = deref_type;
            } else {
                return Err(ToRunnableError::CannotDereferenceTypeNTimes(
                    optr.out(ginfo),
                    *derefs,
                    opt_type,
                ));
            }
        }
        let opt_type_assign = match opt_type.dereference() {
            Some(v) => v,
            None => {
                return Err(ToRunnableError::CannotDereferenceTypeNTimes(
                    optr.out(ginfo),
                    derefs + 1,
                    opt_type,
                ))
            }
        };
        if state.out(ginfo).fits_in(&opt_type_assign, ginfo).is_empty() {
            state.output_to = Some((Box::new(optr), *derefs, is_init));
        } else {
            return Err(ToRunnableError::CannotAssignTo(
                state.out(ginfo),
                opt_type_assign,
            ));
        }
        //
        // if let Some((var_id, var_out)) = linfo.vars.get(opt) {
        //     let out = state.out(ginfo);
        //     let mut var_derefd = var_out.clone();
        //     for _ in 0..*derefs {
        //         var_derefd = if let Some(v) = var_derefd.dereference() {
        //             v
        //         } else {
        //             return Err(ToRunnableError::CannotDereferenceTypeNTimes(
        //                 var_out.clone(),
        //                 *derefs,
        //                 var_derefd,
        //             ));
        //         }
        //     }
        //     let inv_types = out.fits_in(&var_derefd, ginfo);
        //     if !inv_types.is_empty() {
        //         eprintln!("Warn: shadowing variable {opt} because statement's output type {out} does not fit in the original variable's {var_out}. This might become an error in the future, or it might stop shadowing the variiable entirely - for stable scripts, avoid this by giving the variable a different name.");
        //         if *derefs != 0 {
        //             return Err(ToRunnableError::CannotDeclareVariableWithDereference(
        //                 opt.clone(),
        //             ));
        //         }
        //         linfo.vars.insert(opt.clone(), (ginfo.vars, out));
        //         state.output_to = Some((ginfo.vars, 0, true));
        //         ginfo.vars += 1;
        //     } else {
        //         // mutate existing variable
        //         state.output_to = Some((*var_id, *derefs, false));
        //     }
        // } else {
        //     let mut out = state.out(ginfo);
        //     for _ in 0..*derefs {
        //         out = if let Some(v) = out.dereference() {
        //             v
        //         } else {
        //             return Err(ToRunnableError::CannotDereferenceTypeNTimes(
        //                 state.out(ginfo),
        //                 *derefs,
        //                 out,
        //             ));
        //         }
        //     }
        //     linfo.vars.insert(opt.clone(), (ginfo.vars, out));
        //     state.output_to = Some((ginfo.vars, *derefs, true));
        //     ginfo.vars += 1;
        // }
    }
    Ok(state)
}
