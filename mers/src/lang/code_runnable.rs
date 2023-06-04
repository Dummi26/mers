use std::{
    eprintln,
    ops::Deref,
    sync::{Arc, Mutex},
};

use super::{
    builtins::BuiltinFunction,
    global_info::{GSInfo, GlobalScriptInfo},
    to_runnable::ToRunnableError,
    val_data::{VData, VDataEnum},
    val_type::{VSingleType, VType},
};

#[derive(Clone, Debug)]
pub enum RStatementEnum {
    Value(VData),
    Tuple(Vec<RStatement>),
    List(Vec<RStatement>),
    Variable(Arc<Mutex<VData>>, VType, bool),
    FunctionCall(Arc<RFunction>, Vec<RStatement>),
    BuiltinFunctionCall(BuiltinFunction, Vec<RStatement>),
    LibFunctionCall(usize, usize, Vec<RStatement>, VType),
    Block(RBlock),
    If(RStatement, RStatement, Option<RStatement>),
    Loop(RStatement),
    For(RStatement, RStatement, RStatement),
    Switch(RStatement, Vec<(VType, RStatement, RStatement)>, bool),
    Match(Vec<(RStatement, RStatement, RStatement)>),
    IndexFixed(RStatement, usize),
    EnumVariant(usize, RStatement),
}

#[derive(Clone, Debug)]
pub struct RBlock {
    pub statements: Vec<RStatement>,
}
impl RBlock {
    pub fn run(&self, info: &GSInfo) -> VData {
        let mut last = None;
        for statement in &self.statements {
            last = Some(statement.run(info));
        }
        if let Some(v) = last {
            v
        } else {
            VDataEnum::Tuple(vec![]).to()
        }
    }
    pub fn out(&self, info: &GlobalScriptInfo) -> VType {
        if let Some(last) = self.statements.last() {
            last.out(info)
        } else {
            VType {
                types: vec![VSingleType::Tuple(vec![])],
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RFunction {
    pub inputs: Vec<Arc<Mutex<VData>>>,
    pub input_types: Vec<VType>,
    pub input_output_map: Vec<(Vec<VSingleType>, VType)>,
    pub block: RBlock,
}
impl RFunction {
    pub fn run(&self, info: &GSInfo) -> VData {
        self.block.run(info)
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
    pub fn out_vt(&self, input_types: &Vec<VType>, info: &GlobalScriptInfo) -> VType {
        let mut out = VType { types: vec![] };
        for (itype, otype) in self.input_output_map.iter() {
            if itype
                .iter()
                .zip(input_types.iter())
                .all(|(expected, got)| got.contains(expected, info))
            {
                out = out | otype;
            }
        }
        out
    }
    pub fn out_all(&self, info: &GlobalScriptInfo) -> VType {
        self.block.out(info)
    }
    pub fn in_types(&self) -> &Vec<VType> {
        &self.input_types
    }
}

#[derive(Clone, Debug)]
pub struct RStatement {
    // (_, derefs, is_init)
    pub derefs: usize,
    pub output_to: Option<(Box<RStatement>, bool)>,
    statement: Box<RStatementEnum>,
    pub force_output_type: Option<VType>,
}
impl RStatement {
    pub fn run(&self, info: &GSInfo) -> VData {
        let out = self.statement.run(info);
        let mut o = if let Some((v, is_init)) = &self.output_to {
            'init: {
                // // assigns a new VData to the variable's Arc<Mutex<_>>, so that threads which have captured the variable at some point
                // // won't be updated with its new value (is_init is set to true for initializations, such as in a loop - this can happen multiple times, but each should be its own variable with the same name)
                // if *is_init && *derefs == 0 {
                //     Self::assign_to(out, v.run(info), info);
                //     break 'init;
                // }
                let mut val = v.run(info);
                out.assign_to(val, info);
                // val.assign(out);
            }
            VDataEnum::Tuple(vec![]).to()
        } else {
            out
        };
        for _ in 0..self.derefs {
            o = o.deref().expect("couldn't dereference! (run())");
        }
        o
    }
    pub fn out(&self, info: &GlobalScriptInfo) -> VType {
        // `a = b` evaluates to [] (don't change this - cloning is cheap but a = b should NEVER return a boolean because that will make if a = b {} errors way too likely.)
        if self.output_to.is_some() {
            return VType {
                types: vec![VSingleType::Tuple(vec![])],
            };
        }
        if let Some(t) = &self.force_output_type {
            return t.clone();
        }
        let mut o = self.statement.out(info);
        for _ in 0..self.derefs {
            o = o.dereference().expect("can't dereference (out())");
        }
        o
    }
}

impl RStatementEnum {
    pub fn run(&self, info: &GSInfo) -> VData {
        match self {
            Self::Value(v) => v.clone(),
            Self::Tuple(v) => {
                let mut w = vec![];
                for v in v {
                    w.push(v.run(info));
                }
                VDataEnum::Tuple(w).to()
            }
            Self::List(v) => {
                let mut w = vec![];
                let mut out = VType { types: vec![] };
                for v in v {
                    let val = v.run(info);
                    out = out | val.out();
                    w.push(val);
                }
                VDataEnum::List(out, w).to()
            }
            Self::Variable(v, _, is_ref) => {
                if *is_ref {
                    VDataEnum::Reference(v.lock().unwrap().clone_mut()).to()
                } else {
                    v.lock().unwrap().clone_data()
                }
            }
            Self::FunctionCall(func, args) => {
                for (i, input) in func.inputs.iter().enumerate() {
                    input.lock().unwrap().assign(args[i].run(info));
                }
                func.run(info)
            }
            Self::BuiltinFunctionCall(v, args) => v.run(args, info),
            Self::LibFunctionCall(libid, fnid, args, _) => {
                info.libs[*libid].run_fn(*fnid, args.iter().map(|arg| arg.run(info)).collect())
            }
            Self::Block(b) => b.run(info),
            Self::If(c, t, e) => c.run(info).operate_on_data_immut(|v| {
                if let VDataEnum::Bool(v) = v {
                    if *v {
                        t.run(info)
                    } else {
                        if let Some(e) = e {
                            e.run(info)
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    }
                } else {
                    unreachable!()
                }
            }),
            Self::Loop(c) => loop {
                // loops will break if the value matches.
                if let Some(break_val) = c.run(info).matches() {
                    break break_val;
                }
            },
            Self::For(v, c, b) => {
                // matching values also break with value from a for loop.
                let vv = v.run(info);
                let mut in_loop = |c: VData| {
                    c.assign_to(vv.clone_mut(), info);
                    b.run(info)
                };
                let mut iter = c.run(info);
                if let Some(v) = iter.operate_on_data_immut(|c: &VDataEnum| {
                    let mut oval = VDataEnum::Tuple(vec![]).to();
                    match c {
                        VDataEnum::Int(v) => {
                            for i in 0..*v {
                                if let Some(v) = in_loop(VDataEnum::Int(i).to()).matches() {
                                    oval = v;
                                    break;
                                }
                            }
                        }
                        VDataEnum::String(v) => {
                            for ch in v.chars() {
                                if let Some(v) =
                                    in_loop(VDataEnum::String(ch.to_string()).to()).matches()
                                {
                                    oval = v;
                                    break;
                                }
                            }
                        }
                        VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                            for v in v {
                                if let Some(v) = in_loop(v.clone()).matches() {
                                    oval = v;
                                    break;
                                }
                            }
                        }
                        VDataEnum::Function(f) => loop {
                            if let Some(v) = f.run(info).matches() {
                                if let Some(v) = in_loop(v).matches() {
                                    oval = v;
                                    break;
                                }
                            } else {
                                break;
                            }
                        },
                        VDataEnum::Reference(r) => return None,
                        _ => unreachable!(),
                    }
                    Some(oval)
                }) {
                    v
                } else {
                    // loop mutably
                    iter.operate_on_data_mut(|c| match c {
                        VDataEnum::Reference(r) => r.operate_on_data_mut(|c| match c {
                            VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                                for v in v {
                                    if let Some(v) =
                                        in_loop(VDataEnum::Reference(v.clone_mut()).to()).matches()
                                    {
                                        return v;
                                    }
                                }
                                VDataEnum::Tuple(vec![]).to()
                            }
                            _ => unreachable!(),
                        }),
                        _ => unreachable!(),
                    })
                }
            }
            Self::Switch(switch_on, cases, _force) => {
                let switch_on = switch_on.run(info);
                let switch_on_type = switch_on.out();
                let mut out = VDataEnum::Tuple(vec![]).to();
                for (case_type, assign_to, case_action) in cases.iter() {
                    if switch_on_type.fits_in(case_type, info).is_empty() {
                        switch_on.assign_to(assign_to.run(info), info);
                        out = case_action.run(info);
                        break;
                    }
                }
                out
            }
            Self::Match(cases) => 'm: {
                for (case_condition, assign_to, case_action) in cases {
                    // [t] => Some(t), t => Some(t), [] | false => None
                    if let Some(v) = case_condition.run(info).matches() {
                        v.assign_to(assign_to.run(info), info);
                        // let og = { std::mem::replace(&mut *match_on.lock().unwrap(), v) };
                        let res = case_action.run(info);
                        // *match_on.lock().unwrap() = og;
                        break 'm res;
                    }
                }
                VDataEnum::Tuple(vec![]).to()
            }
            Self::IndexFixed(st, i) => st.run(info).get(*i).unwrap(),
            Self::EnumVariant(e, v) => VDataEnum::EnumVariant(*e, Box::new(v.run(info))).to(),
        }
    }
    pub fn out(&self, info: &GlobalScriptInfo) -> VType {
        match self {
            Self::Value(v) => v.out(),
            Self::Tuple(v) => VSingleType::Tuple(v.iter().map(|v| v.out(info)).collect()).into(),
            Self::List(v) => VSingleType::List({
                let mut types = VType { types: vec![] };
                for t in v {
                    types = types | t.out(info);
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
            Self::FunctionCall(f, args) => {
                f.out_vt(&args.iter().map(|v| v.out(info)).collect(), info)
            }
            Self::LibFunctionCall(.., out) => out.clone(),
            Self::Block(b) => b.out(info),
            Self::If(_, a, b) => {
                if let Some(b) = b {
                    a.out(info) | b.out(info)
                } else {
                    a.out(info) | VSingleType::Tuple(vec![]).to()
                }
            }
            Self::Loop(c) => c.out(info).matches().1,
            Self::For(_, _, b) => VSingleType::Tuple(vec![]).to() | b.out(info).matches().1,
            Self::BuiltinFunctionCall(f, args) => {
                f.returns(args.iter().map(|rs| rs.out(info)).collect(), info)
            }
            Self::Switch(switch_on, cases, force) => {
                let switch_on = switch_on.out(info).types;
                let mut might_return_empty = switch_on.is_empty();
                let mut out = if *force {
                    VType::empty()
                } else {
                    VSingleType::Tuple(vec![]).to()
                };
                for switch_on in switch_on {
                    for (_on_type, _assign_to, case) in cases.iter() {
                        out = out | case.out(info);
                    }
                }
                out
            }
            Self::Match(cases) => {
                let mut out = VType::empty();
                let mut can_fail_to_match = true;
                for (condition, _assign_to, action) in cases {
                    out = out | action.out(info);
                    if !condition.out(info).matches().0 {
                        can_fail_to_match = false;
                        break;
                    }
                }
                if can_fail_to_match {
                    out = out | VSingleType::Tuple(vec![]).to()
                }
                out
            }
            Self::IndexFixed(st, i) => st.out(info).get(*i, info).unwrap(),
            Self::EnumVariant(e, v) => VSingleType::EnumVariant(*e, v.out(info)).to(),
        }
    }
    pub fn to(self) -> RStatement {
        RStatement {
            derefs: 0,
            output_to: None,
            statement: Box::new(self),
            force_output_type: None,
        }
    }
}

pub struct RScript {
    main: RFunction,
    info: GSInfo,
}
impl RScript {
    pub fn new(main: RFunction, info: GSInfo) -> Result<Self, ToRunnableError> {
        if main.inputs.len() != 1 {
            return Err(ToRunnableError::MainWrongInput);
        }
        Ok(Self { main, info })
    }
    pub fn run(&self, args: Vec<String>) -> VData {
        let mut vars = vec![];
        vars.push(
            VDataEnum::List(
                VSingleType::String.into(),
                args.into_iter()
                    .map(|v| VDataEnum::String(v).to())
                    .collect(),
            )
            .to(),
        );
        self.main.run(&self.info)
    }
    pub fn info(&self) -> &GSInfo {
        &self.info
    }
}
