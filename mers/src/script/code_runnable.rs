use std::sync::{Arc, Mutex};

use super::{
    builtins::BuiltinFunction,
    global_info::{GSInfo, GlobalScriptInfo},
    to_runnable::ToRunnableError,
    val_data::{VData, VDataEnum},
    val_type::{VSingleType, VType},
};

#[derive(Clone, Debug)]
pub struct RBlock {
    pub statements: Vec<RStatement>,
}
impl RBlock {
    pub fn run(&self, vars: &mut Vec<VData>, info: &GSInfo) -> VData {
        let mut last = None;
        for statement in &self.statements {
            last = Some(statement.run(vars, info));
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
    pub inputs: Vec<usize>,
    pub input_types: Vec<VType>,
    pub input_output_map: Vec<(Vec<VSingleType>, VType)>,
    pub block: RBlock,
}
impl RFunction {
    pub fn run(&self, vars: &mut Vec<VData>, info: &GSInfo) -> VData {
        self.block.run(vars, info)
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
    pub fn out_all(&self, info: &GlobalScriptInfo) -> VType {
        self.block.out(info)
    }
    pub fn in_types(&self) -> &Vec<VType> {
        &self.input_types
    }
}

#[derive(Clone, Debug)]
pub struct RStatement {
    // (_, _, is_init)
    pub output_to: Option<(Box<RStatement>, usize, bool)>,
    statement: Box<RStatementEnum>,
    pub force_output_type: Option<VType>,
}
impl RStatement {
    pub fn run(&self, vars: &mut Vec<VData>, info: &GSInfo) -> VData {
        let out = self.statement.run(vars, info);
        if let Some((v, derefs, is_init)) = &self.output_to {
            let mut val = v.run(vars, info);
            // even if 0 derefs, deref once because it *has* to end on a reference (otherwise writing to it would be unacceptable as the value might not expect to be modified)
            for _ in 0..(derefs + 1) {
                val = match val.inner().deref() {
                    Some(v) => v,
                    None => unreachable!("can't dereference..."),
                };
            }
            val.assign(out.inner());
            VDataEnum::Tuple(vec![]).to()
        } else {
            out
        }
    }
    pub fn out(&self, info: &GlobalScriptInfo) -> VType {
        // `a = b` evaluates to []
        if self.output_to.is_some() {
            return VType {
                types: vec![VSingleType::Tuple(vec![])],
            };
        }
        if let Some(t) = &self.force_output_type {
            return t.clone();
        }
        self.statement.out(info)
    }
}

#[derive(Clone, Debug)]
pub enum RStatementEnum {
    Value(VData),
    Tuple(Vec<RStatement>),
    List(Vec<RStatement>),
    Variable(usize, VType, bool),
    FunctionCall(Arc<RFunction>, Vec<RStatement>),
    BuiltinFunction(BuiltinFunction, Vec<RStatement>),
    LibFunction(usize, usize, Vec<RStatement>, VType),
    Block(RBlock),
    If(RStatement, RStatement, Option<RStatement>),
    Loop(RStatement),
    For(usize, RStatement, RStatement),
    Switch(RStatement, Vec<(VType, RStatement)>),
    Match(usize, Vec<(RStatement, RStatement)>),
    IndexFixed(RStatement, usize),
    EnumVariant(usize, RStatement),
}
impl RStatementEnum {
    pub fn run(&self, vars: &mut Vec<VData>, info: &GSInfo) -> VData {
        match self {
            Self::Value(v) => v.clone(),
            Self::Tuple(v) => {
                let mut w = vec![];
                for v in v {
                    w.push(v.run(vars, info));
                }
                VDataEnum::Tuple(w).to()
            }
            Self::List(v) => {
                let mut w = vec![];
                let mut out = VType { types: vec![] };
                for v in v {
                    let val = v.run(vars, info);
                    out = out | val.out();
                    w.push(val);
                }
                VDataEnum::List(out, w).to()
            }
            Self::Variable(v, _, is_ref) => {
                if *is_ref {
                    // shared mutability (clone_mut)
                    VDataEnum::Reference(vars[*v].clone_mut()).to()
                } else {
                    // Copy on Write (clone)
                    vars[*v].clone()
                }
            }
            Self::FunctionCall(func, args) => {
                for (i, input) in func.inputs.iter().enumerate() {
                    vars[*input] = args[i].run(vars, info);
                }
                func.run(vars, info)
            }
            Self::BuiltinFunction(v, args) => v.run(args, vars, info),
            Self::LibFunction(libid, fnid, args, _) => info.libs[*libid]
                .run_fn(*fnid, args.iter().map(|arg| arg.run(vars, info)).collect()),
            Self::Block(b) => b.run(vars, info),
            Self::If(c, t, e) => {
                if let VDataEnum::Bool(v) = &c.run(vars, info).data().0 {
                    if *v {
                        t.run(vars, info)
                    } else {
                        if let Some(e) = e {
                            e.run(vars, info)
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            Self::Loop(c) => loop {
                // loops will break if the value matches.
                if let Some(break_val) = c.run(vars, info).inner().matches() {
                    break break_val;
                }
            },
            Self::For(v, c, b) => {
                // matching values also break with value from a for loop.
                let c = c.run(vars, info);
                let mut vars = vars.clone();
                let in_loop = |vars: &mut Vec<VData>, c| {
                    vars[*v] = c;
                    b.run(vars, info)
                };

                let mut oval = VDataEnum::Tuple(vec![]).to();
                match &c.data().0 {
                    VDataEnum::Int(v) => {
                        for i in 0..*v {
                            if let Some(v) =
                                in_loop(&mut vars, VDataEnum::Int(i).to()).inner().matches()
                            {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::String(v) => {
                        for ch in v.chars() {
                            if let Some(v) =
                                in_loop(&mut vars, VDataEnum::String(ch.to_string()).to())
                                    .inner()
                                    .matches()
                            {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                        for v in v {
                            if let Some(v) = in_loop(&mut vars, v.clone()).inner().matches() {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::Function(f) => loop {
                        if let Some(v) = f.run(&mut vars, info).inner().matches() {
                            if let Some(v) = in_loop(&mut vars, v).inner().matches() {
                                oval = v;
                                break;
                            }
                        } else {
                            break;
                        }
                    },
                    _ => unreachable!(),
                }
                oval
            }
            Self::Switch(switch_on, cases) => {
                let switch_on = switch_on.run(vars, info);
                let switch_on_type = switch_on.out();
                let mut out = VDataEnum::Tuple(vec![]).to();
                for (case_type, case_action) in cases.iter() {
                    if switch_on_type.fits_in(case_type, info).is_empty() {
                        out = case_action.run(vars, info);
                        break;
                    }
                }
                out
            }
            Self::Match(match_on, cases) => 'm: {
                for (case_condition, case_action) in cases {
                    // [t] => Some(t), t => Some(t), [] | false => None
                    if let Some(v) = case_condition.run(vars, info).inner().matches() {
                        let og = { std::mem::replace(&mut vars[*match_on], v) };
                        let res = case_action.run(vars, info);
                        vars[*match_on] = og;
                        break 'm res;
                    }
                }
                VDataEnum::Tuple(vec![]).to()
            }
            Self::IndexFixed(st, i) => st.run(vars, info).get(*i).unwrap(),
            Self::EnumVariant(e, v) => VDataEnum::EnumVariant(*e, Box::new(v.run(vars, info))).to(),
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
            Self::FunctionCall(f, args) => f.out_vt(&args.iter().map(|v| v.out(info)).collect()),
            Self::LibFunction(.., out) => out.clone(),
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
            Self::BuiltinFunction(f, args) => {
                f.returns(args.iter().map(|rs| rs.out(info)).collect(), info)
            }
            Self::Switch(switch_on, cases) => {
                let switch_on = switch_on.out(info).types;
                let mut might_return_empty = switch_on.is_empty();
                let mut out = VType { types: vec![] }; // if nothing is executed
                for switch_on in switch_on {
                    let switch_on = switch_on.to();
                    'search: {
                        for (on_type, case) in cases.iter() {
                            if switch_on.fits_in(&on_type, info).is_empty() {
                                out = out | case.out(info);
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
                    out = out | case.1.out(info);
                }
                out
            }
            Self::IndexFixed(st, i) => st.out(info).get(*i, info).unwrap(),
            Self::EnumVariant(e, v) => VSingleType::EnumVariant(*e, v.out(info)).to(),
        }
    }
    pub fn to(self) -> RStatement {
        RStatement {
            output_to: None,
            statement: Box::new(self),
            force_output_type: None,
        }
    }
}

#[derive(Debug)]
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
        let mut vars = Vec::with_capacity(self.info.vars);
        vars.push(
            VDataEnum::List(
                VSingleType::String.into(),
                args.into_iter()
                    .map(|v| VDataEnum::String(v).to())
                    .collect(),
            )
            .to(),
        );
        for _i in 1..self.info.vars {
            vars.push(VDataEnum::Tuple(vec![]).to());
        }
        self.main.run(&mut vars, &self.info)
    }
    pub fn info(&self) -> &GSInfo {
        &self.info
    }
}
