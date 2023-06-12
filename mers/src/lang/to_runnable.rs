use core::panic;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::lang::{
    global_info::GlobalScriptInfo,
    val_data::{VData, VDataEnum},
    val_type::{VSingleType, VType},
};

use super::{
    builtins::BuiltinFunction,
    code_macro::Macro,
    code_parsed::{SBlock, SFunction, SStatement, SStatementEnum},
    code_runnable::{RBlock, RFunction, RScript, RStatement, RStatementEnum},
    fmtgs::FormatGs,
    global_info::GSInfo,
};

pub enum ToRunnableError {
    MainWrongInput,
    UseOfUndefinedVariable(String),
    UseOfUndefinedFunction(String),
    UnknownType(String),
    CannotDereferenceTypeNTimes(VType, usize, VType),
    FunctionWrongArgs(String, Vec<Arc<RFunction>>, Vec<VType>),
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
//  - Don't use {} to format, use .fmtgs(f, info, form, file) instead!
//  - Show location in code where the error was found
impl Display for ToRunnableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmtgs(f, None, &mut super::fmtgs::FormatInfo::default(), None)
    }
}
impl FormatGs for ToRunnableError {
    fn fmtgs(
        &self,
        f: &mut std::fmt::Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        match self {
            Self::MainWrongInput => write!(
                f,
                "Main function had the wrong input. This is a bug and should never happen."
            ),
            Self::UseOfUndefinedVariable(v) => {
                write!(f, "Cannot use variable \"{v}\" as it isn't defined (yet?).")
            }
            Self::UseOfUndefinedFunction(v) => {
                write!(f, "Cannot use function \"{v}\" as it isn't defined (yet?).")
            }
            Self::UnknownType(name) => write!(f, "Unknown type \"{name}\"."),
            Self::CannotDereferenceTypeNTimes(og_type, derefs_wanted, last_valid_type) => {
                write!(f, "Cannot dereference type ")?;
                og_type.fmtgs(f, info, form, file)?;
                write!(f, " {derefs_wanted} times (stopped at ")?;
                last_valid_type.fmtgs(f, info, form, file)?;
                write!(f, ")")?;
                Ok(())
            }
            Self::FunctionWrongArgs(fn_name, possible_fns, given_types) => {
                write!(f, "Wrong args for function \"{fn_name}\": Found (")?;
                for (i, t) in given_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    t.fmtgs(f, info, form, file)?;
                }
                write!(f, "), but valid are only ")?;
                for (i, func) in possible_fns.iter().enumerate() {
                    if i != 0 {
                        if i + 1 == possible_fns.len() {
                            write!(f, ", and ")?;
                        } else {
                            write!(f, ", ")?;
                        }
                    }
                    VDataEnum::Function(Arc::clone(func)).fmtgs(f, info, form, file)?;
                }
                write!(f, ".")?;
                Ok(())
            }
            Self::InvalidType {
                expected,
                found,
                problematic,
            } => {
                write!(f, "Invalid type: Expected ")?;
                expected.fmtgs(f, info, form, file)?;
                write!(f, " but found ")?;
                found.fmtgs(f, info, form, file)?;
                write!(f, ", which includes ")?;
                problematic.fmtgs(f, info, form, file)?;
                write!(f, " which is not covered.")?;
                Ok(())
            }
            Self::CaseForceButTypeNotCovered(v) => {
                write!(
                    f,
                    "Switch! statement, but not all types covered. Types to cover: "
                )?;
                v.fmtgs(f, info, form, file)?;
                Ok(())
            }
            Self::MatchConditionInvalidReturn(v) => {
                write!(f, "match statement condition returned ")?;
                v.fmtgs(f, info, form, file)?;
                write!(f, ", which is not necessarily a tuple of size 0 to 1.")?;
                Ok(())
            }
            Self::NotIndexableFixed(t, i) => {
                write!(f, "Cannot use fixed-index {i} on type ")?;
                t.fmtgs(f, info, form, file)?;
                write!(f, ".")?;
                Ok(())
            }
            Self::WrongInputsForBuiltinFunction(_builtin, builtin_name, args) => {
                write!(
                    f,
                    "Wrong arguments for builtin function \"{}\":",
                    builtin_name
                )?;
                for arg in args {
                    write!(f, " ")?;
                    arg.fmtgs(f, info, form, file)?;
                }
                write!(f, ".")
            }
            Self::WrongArgsForLibFunction(name, args) => {
                write!(f, "Wrong arguments for library function {}:", name)?;
                for arg in args {
                    write!(f, " ")?;
                    arg.fmtgs(f, info, form, file)?;
                }
                write!(f, ".")
            }
            Self::CannotAssignTo(val, target) => {
                write!(f, "Cannot assign type ")?;
                val.fmtgs(f, info, form, file)?;
                write!(f, " to ")?;
                target.fmtgs(f, info, form, file)?;
                write!(f, ".")?;
                Ok(())
            }
            Self::ForLoopContainerHasNoInnerTypes => {
                write!(f, "For loop: container had no inner types, cannot iterate.")
            }
            Self::StatementRequiresOutputTypeToBeAButItActuallyOutputsBWhichDoesNotFitInA(
                required,
                real,
                problematic,
            ) => {
                write!(f, "the statement requires its output type to be ")?;
                required.fmtgs(f, info, form, file)?;
                write!(f, ", but its real output type is ")?;
                real.fmtgs(f, info, form, file)?;
                write!(
                    f,
                    ", which doesn't fit in the required type because of the problematic types "
                )?;
                problematic.fmtgs(f, info, form, file)?;
                write!(f, ".")?;
                Ok(())
            }
        }
    }
}

// Local, used to keep local variables separated
#[derive(Clone)]
struct LInfo {
    vars: HashMap<String, Arc<Mutex<(VData, VType)>>>,
    fns: HashMap<String, Vec<Arc<RFunction>>>,
}

pub fn to_runnable(
    s: SFunction,
    mut ginfo: GlobalScriptInfo,
) -> Result<RScript, (ToRunnableError, GSInfo)> {
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
        let var = Arc::new(Mutex::new((VData::new_placeholder(), itype.clone())));
        linfo.vars.insert(iname.clone(), Arc::clone(&var));
        input_vars.push(var);
        input_types.push(itype);
    }
    // set the types to all possible types (get_all_functions sets the types to one single type to get the return type of the block for that case)
    for (varid, vartype) in s.inputs.iter().zip(input_types.iter()) {
        linfo.vars.get(&varid.0).unwrap().lock().unwrap().1 = vartype.clone();
    }
    let mut o = RFunction {
        out_map: vec![],
        inputs: input_vars,
        input_types,
        statement: statement(&s.statement, ginfo, &mut linfo.clone())?,
    };
    o.out_map = {
        let mut map = vec![];
        let mut indices: Vec<_> = o.input_types.iter().map(|_| 0).collect();
        // like counting: advance first index, when we reach the end, reset to zero and advance the next index, ...
        loop {
            let mut current_types = Vec::with_capacity(o.input_types.len());
            let mut adv = true;
            let mut was_last = o.input_types.is_empty();
            for i in 0..o.input_types.len() {
                current_types.push(match o.input_types[i].types.get(indices[i]) {
                    Some(v) => v.clone().to(),
                    None => VType::empty(),
                });
                if adv {
                    if indices[i] + 1 < o.input_types[i].types.len() {
                        indices[i] += 1;
                        adv = false;
                    } else {
                        indices[i] = 0;
                        // we just reset the last index back to 0 - if we don't break
                        // from the loop, we will just start all over again.
                        if i + 1 == o.input_types.len() {
                            was_last = true;
                        }
                    }
                }
            }
            let out = o.out_by_statement(&current_types, &ginfo);
            map.push((current_types, out));
            if was_last {
                break map;
            }
        }
    };
    Ok(o)
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
        VSingleType::Any
        | VSingleType::Bool
        | VSingleType::Int
        | VSingleType::Float
        | VSingleType::String => (),
        VSingleType::Tuple(v) => {
            for t in v {
                stypes(t, ginfo)?;
            }
        }
        VSingleType::List(t) => stypes(t, ginfo)?,
        VSingleType::Reference(t) => stypes(t, ginfo)?,
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
                    stypes(i, ginfo)?;
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
    statement_adv(s, ginfo, linfo, &mut None)
}
fn statement_adv(
    s: &SStatement,
    ginfo: &mut GlobalScriptInfo,
    linfo: &mut LInfo,
    // if Some((t, is_init)), the statement creates by this function is the left side of an assignment, meaning it can create variables. t is the type that will be assigned to it.
    to_be_assigned_to: &mut Option<(VType, &mut bool)>,
) -> Result<RStatement, ToRunnableError> {
    // eprintln!("TR : {}", s);
    // if let Some(t) = &to_be_assigned_to {
    //     eprintln!(" --> {}", t.0);
    // }
    let mut state = match &*s.statement {
        SStatementEnum::Value(v) => RStatementEnum::Value(v.clone()).to(),
        SStatementEnum::Tuple(v) | SStatementEnum::List(v) => {
            let mut w = Vec::with_capacity(v.len());
            let mut prev = None;
            for (i, v) in v.iter().enumerate() {
                if let Some(t) = to_be_assigned_to {
                    let out_t = if let Some(p) = &prev { p } else { &t.0 };
                    let inner_t = if let Some(v) = out_t.get_always(i, ginfo) {
                        v
                    } else {
                        panic!("cannot assign: cannot get_always({i}) on type {}.", out_t);
                    };
                    let p = std::mem::replace(&mut t.0, inner_t);
                    if prev.is_none() {
                        prev = Some(p);
                    }
                };
                w.push(statement_adv(v, ginfo, linfo, to_be_assigned_to)?);
            }
            if let (Some(t), Some(prev)) = (to_be_assigned_to, prev) {
                t.0 = prev;
            }
            if let SStatementEnum::List(_) = &*s.statement {
                RStatementEnum::List(w)
            } else {
                RStatementEnum::Tuple(w)
            }
            .to()
        }
        SStatementEnum::Variable(v, is_ref) => {
            let existing_var = linfo.vars.get(v);
            let is_init_force = if let Some(v) = &to_be_assigned_to {
                *v.1
            } else {
                false
            };
            // we can't assign to a variable that doesn't exist yet -> create a new one
            if is_init_force
                || (existing_var.is_none() && ginfo.to_runnable_automatic_initialization)
            {
                // if to_be_assigned_to is some (-> this is on the left side of an assignment), create a new variable. else, return an error.
                if let Some((t, is_init)) = to_be_assigned_to {
                    **is_init = true;
                    #[cfg(not(debug_assertions))]
                    let var = VData::new_placeholder();
                    #[cfg(debug_assertions)]
                    let var = VData::new_placeholder_with_name(v.to_owned());
                    let var_arc = Arc::new(Mutex::new((var, t.clone())));
                    linfo.vars.insert(v.to_owned(), Arc::clone(&var_arc));
                    RStatementEnum::Variable(var_arc, true)
                } else {
                    return Err(ToRunnableError::UseOfUndefinedVariable(v.clone()));
                }
            } else if let Some(var) = existing_var {
                RStatementEnum::Variable(Arc::clone(&var), *is_ref)
            } else {
                return Err(ToRunnableError::UseOfUndefinedVariable(v.clone()));
            }
            .to()
        }
        SStatementEnum::FunctionCall(v, args) => {
            let mut rargs = Vec::with_capacity(args.len());
            for arg in args.iter() {
                rargs.push(statement(arg, ginfo, linfo)?);
            }
            let arg_types: Vec<_> = rargs.iter().map(|v| v.out(ginfo)).collect();
            fn check_fn_args(
                arg_types: &Vec<VType>,
                func: &RFunction,
                ginfo: &GlobalScriptInfo,
            ) -> bool {
                func.inputs.len() == arg_types.len()
                    && func
                        .inputs
                        .iter()
                        .zip(arg_types.iter())
                        .all(|(fn_in, arg)| arg.fits_in(&fn_in.lock().unwrap().1, ginfo).is_empty())
            }
            if let Some(funcs) = linfo.fns.get(v) {
                'find_func: {
                    for func in funcs.iter().rev() {
                        if check_fn_args(&arg_types, &func, ginfo) {
                            break 'find_func RStatementEnum::FunctionCall(
                                Arc::clone(&func),
                                rargs,
                            );
                        }
                    }
                    return Err(ToRunnableError::FunctionWrongArgs(
                        v.to_owned(),
                        funcs.iter().map(|v| Arc::clone(v)).collect(),
                        arg_types,
                    ));
                }
            } else {
                if let Some(builtin) = BuiltinFunction::get(v) {
                    let arg_types = rargs.iter().map(|v| v.out(ginfo)).collect();
                    if builtin.can_take(&arg_types, ginfo) {
                        RStatementEnum::BuiltinFunctionCall(builtin, rargs)
                    } else {
                        return Err(ToRunnableError::WrongInputsForBuiltinFunction(
                            builtin,
                            v.to_string(),
                            arg_types,
                        ));
                    }
                } else {
                    // LIBRARY FUNCTION?
                    if let Some((libid, fnid)) = ginfo.lib_fns.get(v) {
                        let lib = &ginfo.libs[*libid];
                        let libfn = &lib.registered_fns[*fnid];
                        let mut empty = true;
                        let fn_out =
                            libfn
                                .1
                                .iter()
                                .fold(VType::empty(), |mut t, (fn_in, fn_out)| {
                                    if fn_in.len() == arg_types.len()
                                        && fn_in.iter().zip(arg_types.iter()).all(|(fn_in, arg)| {
                                            arg.fits_in(fn_in, ginfo).is_empty()
                                        })
                                    {
                                        empty = false;
                                        t.add_typesr(fn_out, ginfo);
                                    }
                                    t
                                });
                        if empty {
                            return Err(ToRunnableError::WrongArgsForLibFunction(
                                v.to_owned(),
                                arg_types,
                            ));
                        }
                        RStatementEnum::LibFunctionCall(*libid, *fnid, rargs, fn_out.clone())
                    } else {
                        return Err(ToRunnableError::UseOfUndefinedFunction(v.clone()));
                    }
                }
            }
        }
        .to(),
        SStatementEnum::FunctionDefinition(name, f) => {
            let f = Arc::new(function(f, ginfo, linfo.clone())?);
            if let Some(name) = name {
                // named function => add to global functions
                let f = Arc::clone(&f);
                if let Some(vec) = linfo.fns.get_mut(name) {
                    vec.push(f);
                } else {
                    linfo.fns.insert(name.clone(), vec![f]);
                }
            }
            RStatementEnum::Value(VDataEnum::Function(f).to()).to()
        }
        SStatementEnum::Block(b) => RStatementEnum::Block(block(&b, ginfo, linfo.clone())?).to(),
        SStatementEnum::If(c, t, e) => RStatementEnum::If(
            {
                let condition = statement(&c, ginfo, linfo)?;
                let out = condition.out(ginfo).fits_in(
                    &VType {
                        types: vec![VSingleType::Bool],
                    },
                    ginfo,
                );
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
        )
        .to(),
        SStatementEnum::Loop(c) => RStatementEnum::Loop(statement(&c, ginfo, linfo)?).to(),
        SStatementEnum::For(v, c, b) => {
            let mut linfo = linfo.clone();
            let container = statement(&c, ginfo, &mut linfo)?;
            let inner = container.out(ginfo).inner_types_for_iters(ginfo);
            if inner.types.is_empty() {
                return Err(ToRunnableError::ForLoopContainerHasNoInnerTypes);
            }
            let assign_to = statement_adv(v, ginfo, &mut linfo, &mut Some((inner, &mut true)))?;
            let block = statement(&b, ginfo, &mut linfo)?;
            let o = RStatementEnum::For(assign_to, container, block);
            o.to()
        }

        SStatementEnum::Switch(switch_on, cases, force) => {
            let mut ncases = Vec::with_capacity(cases.len());
            let switch_on = statement(switch_on, ginfo, linfo)?;
            let og_type = switch_on.out(ginfo);
            let mut covered_types = VType::empty();
            for (case_type, case_assign_to, case_action) in cases.iter() {
                let mut linfo = linfo.clone();
                let case_type = {
                    let mut v = case_type.clone();
                    stypes(&mut v, ginfo)?;
                    v
                };
                covered_types.add_typesr(&case_type, ginfo);
                ncases.push((
                    case_type.clone(),
                    statement_adv(
                        case_assign_to,
                        ginfo,
                        &mut linfo,
                        &mut Some((case_type, &mut true)),
                    )?,
                    statement(case_action, ginfo, &mut linfo)?,
                ));
            }
            if *force {
                let types_not_covered = og_type.fits_in(&covered_types, ginfo);
                if !types_not_covered.is_empty() {
                    return Err(ToRunnableError::CaseForceButTypeNotCovered(VType {
                        types: types_not_covered,
                    }));
                }
            }
            RStatementEnum::Switch(switch_on, ncases, *force).to()
        }
        SStatementEnum::Match(cases) => {
            let _ncases: Vec<(RStatement, RStatement, RStatement)> =
                Vec::with_capacity(cases.len());
            let mut ncases = Vec::with_capacity(cases.len());
            let mut out_type = VType::empty();
            let may_not_match = true;
            for (condition, assign_to, action) in cases.iter() {
                let mut linfo = linfo.clone();
                let condition = statement(condition, ginfo, &mut linfo)?;
                let (can_fail, matches) = condition.out(ginfo).matches(ginfo);
                let assign_to = statement_adv(
                    assign_to,
                    ginfo,
                    &mut linfo,
                    &mut Some((matches, &mut true)),
                )?;
                let action = statement(action, ginfo, &mut linfo)?;
                ncases.push((condition, assign_to, action));
                if !can_fail {
                    break;
                }
            }
            if may_not_match {
                out_type.add_type(VSingleType::Tuple(vec![]), ginfo);
            }

            RStatementEnum::Match(ncases).to()
        }

        SStatementEnum::IndexFixed(st, i) => {
            let st = statement(st, ginfo, linfo)?;
            if st.out(ginfo).get_always(*i, ginfo).is_some() {
                RStatementEnum::IndexFixed(st, *i).to()
            } else {
                return Err(ToRunnableError::NotIndexableFixed(st.out(ginfo), *i));
            }
        }
        SStatementEnum::EnumVariant(variant, s) => RStatementEnum::EnumVariant(
            {
                if let Some(v) = ginfo.enum_variants.get(variant) {
                    *v
                } else {
                    let v = ginfo.enum_variants.len();
                    ginfo.enum_variants.insert(variant.clone(), v);
                    v
                }
            },
            statement(s, ginfo, linfo)?,
        )
        .to(),
        SStatementEnum::TypeDefinition(name, t) => {
            // insert to name map has to happen before stypes()
            ginfo
                .custom_type_names
                .insert(name.to_lowercase(), ginfo.custom_types.len());
            let mut t = t.to_owned();
            stypes(&mut t, ginfo)?;
            ginfo.custom_types.push(t);
            RStatementEnum::Value(VDataEnum::Tuple(vec![]).to()).to()
        }
        SStatementEnum::Macro(m) => match m {
            Macro::StaticMers(val) => RStatementEnum::Value(val.clone()).to(),
        },
    };
    state.derefs = s.derefs;
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
    if let Some((opt, is_init)) = &s.output_to {
        // if false, may be changed to true by statement_adv
        let mut is_init = *is_init;
        let optr = statement_adv(
            opt,
            ginfo,
            linfo,
            &mut Some((state.out(ginfo), &mut is_init)),
        )?;
        state.output_to = Some((Box::new(optr), is_init));
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
