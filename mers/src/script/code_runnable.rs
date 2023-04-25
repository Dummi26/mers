use std::sync::{Arc, Mutex};

use super::{
    builtins::BuiltinFunction,
    global_info::GSInfo,
    to_runnable::ToRunnableError,
    val_data::{VData, VDataEnum},
    val_type::{VSingleType, VType},
};

type Am<T> = Arc<Mutex<T>>;
fn am<T>(i: T) -> Am<T> {
    Arc::new(Mutex::new(i))
}

#[derive(Clone, Debug)]
pub struct RBlock {
    pub statements: Vec<RStatement>,
}
impl RBlock {
    pub fn run(&self, vars: &Vec<Am<VData>>, info: &GSInfo) -> VData {
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
    pub fn run(&self, vars: &Vec<Am<VData>>, info: &GSInfo) -> VData {
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
    pub fn out_all(&self) -> VType {
        self.block.out()
    }
    pub fn in_types(&self) -> &Vec<VType> {
        &self.input_types
    }
}

#[derive(Clone, Debug)]
pub struct RStatement {
    pub output_to: Option<(usize, usize)>,
    statement: Box<RStatementEnum>,
    pub force_output_type: Option<VType>,
}
impl RStatement {
    pub fn run(&self, vars: &Vec<Am<VData>>, info: &GSInfo) -> VData {
        let out = self.statement.run(vars, info);
        if let Some((v, derefs)) = self.output_to {
            let mut val = vars[v].clone();
            for _ in 0..derefs {
                let v = if let VDataEnum::Reference(v) = &val.lock().unwrap().data {
                    v.clone()
                } else {
                    unreachable!("dereferencing something that isn't a reference in assignment")
                };
                val = v;
            }
            *val.lock().unwrap() = out;
            VDataEnum::Tuple(vec![]).to()
        } else {
            out
        }
    }
    pub fn out(&self) -> VType {
        // `a = b` evaluates to []
        if self.output_to.is_some() {
            return VType {
                types: vec![VSingleType::Tuple(vec![])],
            };
        }
        if let Some(t) = &self.force_output_type {
            return t.clone();
        }
        self.statement.out()
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
    pub fn run(&self, vars: &Vec<Am<VData>>, info: &GSInfo) -> VData {
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
                    VDataEnum::Reference(vars[*v].clone()).to()
                } else {
                    vars[*v].lock().unwrap().clone()
                }
            }
            Self::FunctionCall(func, args) => {
                for (i, input) in func.inputs.iter().enumerate() {
                    *vars[*input].lock().unwrap() = args[i].run(vars, info);
                }
                func.run(vars, info)
            }
            Self::BuiltinFunction(v, args) => v.run(args, vars, info),
            Self::LibFunction(libid, fnid, args, _) => info.libs[*libid]
                .run_fn(*fnid, &args.iter().map(|arg| arg.run(vars, info)).collect()),
            Self::Block(b) => b.run(vars, info),
            Self::If(c, t, e) => {
                if let VDataEnum::Bool(v) = c.run(vars, info).data {
                    if v {
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
                // While loops will break if the value matches.
                if let Some(break_val) = c.run(vars, info).data.matches() {
                    break break_val;
                }
            },
            Self::For(v, c, b) => {
                // matching values also break with value from a for loop.
                let c = c.run(vars, info);
                let mut vars = vars.clone();
                let in_loop = |vars: &mut Vec<Arc<Mutex<VData>>>, c| {
                    vars[*v] = Arc::new(Mutex::new(c));
                    b.run(&vars, info)
                };

                let mut oval = VDataEnum::Tuple(vec![]).to();
                match c.data {
                    VDataEnum::Int(v) => {
                        for i in 0..v {
                            if let Some(v) =
                                in_loop(&mut vars, VDataEnum::Int(i).to()).data.matches()
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
                                    .data
                                    .matches()
                            {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                        for v in v {
                            if let Some(v) = in_loop(&mut vars, v).data.matches() {
                                oval = v;
                                break;
                            }
                        }
                    }
                    VDataEnum::Function(f) => loop {
                        if let Some(v) = f.run(&vars, info).data.matches() {
                            if let Some(v) = in_loop(&mut vars, v).data.matches() {
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
                    if switch_on_type.fits_in(case_type).is_empty() {
                        out = case_action.run(vars, info);
                        break;
                    }
                }
                out
            }
            Self::Match(match_on, cases) => 'm: {
                for (case_condition, case_action) in cases {
                    // [t] => Some(t), t => Some(t), [] | false => None
                    if let Some(v) = case_condition.run(vars, info).data.matches() {
                        let og = { std::mem::replace(&mut *vars[*match_on].lock().unwrap(), v) };
                        let res = case_action.run(vars, info);
                        *vars[*match_on].lock().unwrap() = og;
                        break 'm res;
                    }
                }
                VDataEnum::Tuple(vec![]).to()
            }
            Self::IndexFixed(st, i) => st.run(vars, info).get(*i).unwrap(),
            Self::EnumVariant(e, v) => VDataEnum::EnumVariant(*e, Box::new(v.run(vars, info))).to(),
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
                    a.out() | VSingleType::Tuple(vec![]).to()
                }
            }
            Self::Loop(c) => c.out().matches().1,
            Self::For(_, _, b) => VSingleType::Tuple(vec![]).to() | b.out().matches().1,
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
            force_output_type: None,
        }
    }
}

pub struct RScript {
    main: RFunction,
    vars: usize,
    info: GSInfo,
}
impl RScript {
    pub fn new(main: RFunction, vars: usize, info: GSInfo) -> Result<Self, ToRunnableError> {
        if main.inputs.len() != 1 {
            return Err(ToRunnableError::MainWrongInput);
        }
        Ok(Self { main, vars, info })
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
        self.main.run(&vars, &self.info)
    }
}
