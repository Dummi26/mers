use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    data::{self, function::FunctionT, Data, MersData, MersType, Type},
    program::{
        self,
        run::{CheckError, CheckInfo},
    },
};

use super::Config;

impl Config {
    /// Adds functions to deal with iterables
    /// `iter: fn` executes a function once for each element of the iterable
    /// `map: fn` maps each value in the iterable to a new one by applying a transformation function
    /// `filter: fn` filters the iterable by removing all elements where the filter function doesn't return true
    /// `filter_map: fn` combines filter and map. requires that the function returns ()/(t).
    /// `enumerate: fn` transforms an iterator over T into one over (Int, T), where Int is the index of the element
    pub fn with_iters(self) -> Self {
        self.add_var(
            "for_each".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for a in &a.types {
                        if let Some(tuple) = a.as_any().downcast_ref::<data::tuple::TupleT>() {
                            if let (Some(v), Some(f)) = (tuple.0.get(0), tuple.0.get(1)) {
                                if let (Some(iter), Some(f)) = (
                                    v.iterable(),
                                    f.types
                                        .iter()
                                        .map(|t| {
                                            t.as_any().downcast_ref::<data::function::FunctionT>()
                                        })
                                        .collect::<Option<Vec<_>>>(),
                                ) {
                                    for f in f {
                                        let ret = f.0(&iter)?;
                                        if !ret.is_zero_tuple() {
                                            return Err(format!("for_each function must return (), not {ret}").into());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "for_each called on tuple not containing iterable and function: {v} is {}",
                                        if v.iterable().is_some() { "iterable" } else { "not iterable" },
                                    ).into());
                                }
                            } else {
                                return Err(format!(
                                    "for_each called on tuple with len < 2"
                                ).into());
                            }
                        } else {
                            return Err(format!("for_each called on non-tuple").into());
                        }
                    }
                    Ok(Type::empty_tuple())
                }),
                run: Arc::new(|a, _i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                            if let (Some(iter), Some(f)) = (
                                v.get().iterable(),
                                f.get().as_any().downcast_ref::<data::function::Function>(),
                            ) {
                                for v in iter {
                                    f.run(v);
                                }
                                Data::empty_tuple()
                            } else {
                                unreachable!(
                                    "for_each called on tuple not containing iterable and function"
                                )
                            }
                        } else {
                            unreachable!("for_each called on tuple with len < 2")
                        }
                    } else {
                        unreachable!("for_each called on non-tuple")
                    }
                }),
            }),
        )
        .add_var(
            "map".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                        iter_out(a, "map", |f| ItersT::Map(f.clone()))
                    }),
                run: Arc::new(|a, _i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                            if let Some(f) =
                                f.get().as_any().downcast_ref::<data::function::Function>()
                            {
                                Data::new(Iter(Iters::Map(Clone::clone(f)), v.clone()))
                            } else {
                                unreachable!("iter called on tuple not containing function")
                            }
                        } else {
                            unreachable!("iter called on tuple with len < 2")
                        }
                    } else {
                        unreachable!("iter called on non-tuple")
                    }
                }),
            }),
        )
        .add_var(
            "filter".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                        iter_out(a, "filter", |f| ItersT::Filter(f.clone()))
                    }),
                run: Arc::new(|a, _i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                            if let Some(f) =
                                f.get().as_any().downcast_ref::<data::function::Function>()
                            {
                                Data::new(Iter(Iters::Filter(Clone::clone(f)), v.clone()))
                            } else {
                                unreachable!("iter called on tuple not containing function")
                            }
                        } else {
                            unreachable!("iter called on tuple with len < 2")
                        }
                    } else {
                        unreachable!("iter called on non-tuple")
                    }
                }),
            }),
        )
        .add_var(
            "filter_map".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                        iter_out(a, "filter_map", |f| ItersT::FilterMap(f.clone()))
                    }),
                run: Arc::new(|a, _i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                            if let Some(f) =
                                f.get().as_any().downcast_ref::<data::function::Function>()
                            {
                                Data::new(Iter(Iters::FilterMap(Clone::clone(f)), v.clone()))
                            } else {
                                unreachable!("iter called on tuple not containing function")
                            }
                        } else {
                            unreachable!("iter called on tuple with len < 2")
                        }
                    } else {
                        unreachable!("iter called on non-tuple")
                    }
                }),
            }),
        )
        .add_var(
            "enumerate".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let data = if let Some(a) = a.iterable() {
                        a
                    } else {
                        return Err(format!("cannot call enumerate on non-iterable type {a}.").into());
                    };
                    Ok(Type::new(IterT::new(ItersT::Enumerate, data)?))
                }),
                run: Arc::new(|a, _i| Data::new(Iter(Iters::Enumerate, a.clone()))),
            }),
        )
    }
}

fn iter_out(a: &Type, name: &str, func: impl Fn(&FunctionT) -> ItersT) -> Result<Type, CheckError> {
    let mut out = Type::empty();
    for t in a.types.iter() {
        if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
            if t.0.len() != 2 {
                return Err(format!("cannot call {name} on tuple where len != 2").into());
            }
            if let Some(v) = t.0[0].iterable() {
                for f in t.0[1].types.iter() {
                    if let Some(f) = f.as_any().downcast_ref::<data::function::FunctionT>() {
                        out.add(Arc::new(IterT::new(func(f), v.clone())?));
                    } else {
                        return Err(format!("cannot call {name} on tuple that isn't (_, function): got {} instead of function as part of {a}", t.0[1]).into());
                    }
                }
            } else {
                return Err(format!(
                    "cannot call {name} on non-iterable type {t}, which is part of {a}."
                )
                .into());
            }
        }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
pub enum Iters {
    Map(data::function::Function),
    Filter(data::function::Function),
    FilterMap(data::function::Function),
    Enumerate,
}
#[derive(Clone, Debug)]
pub enum ItersT {
    Map(data::function::FunctionT),
    Filter(data::function::FunctionT),
    FilterMap(data::function::FunctionT),
    Enumerate,
}
#[derive(Clone, Debug)]
pub struct Iter(Iters, Data);
#[derive(Clone, Debug)]
pub struct IterT(ItersT, Type, Type);
impl MersData for Iter {
    fn is_eq(&self, _other: &dyn MersData) -> bool {
        false
    }
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        Some(match &self.0 {
            Iters::Map(f) => {
                let f = Clone::clone(f);
                Box::new(self.1.get().iterable()?.map(move |v| f.run(v)))
            }
            Iters::Filter(f) => {
                let f = Clone::clone(f);
                Box::new(self.1.get().iterable()?.filter(move |v| {
                    f.run(v.clone())
                        .get()
                        .as_any()
                        .downcast_ref::<data::bool::Bool>()
                        .is_some_and(|b| b.0)
                }))
            }
            Iters::FilterMap(f) => {
                let f = Clone::clone(f);
                Box::new(
                    self.1
                        .get()
                        .iterable()?
                        .filter_map(move |v| f.run(v).one_tuple_content()),
                )
            }
            Iters::Enumerate => Box::new(self.1.get().iterable()?.enumerate().map(|(i, v)| {
                Data::new(data::tuple::Tuple(vec![
                    Data::new(data::int::Int(i as _)),
                    v,
                ]))
            })),
        })
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> data::Type {
        Type::new(IterT::new(self.0.as_type(), self.1.get().as_type()).unwrap())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl IterT {
    pub fn new(iter: ItersT, data: Type) -> Result<Self, CheckError> {
        let t = match &iter {
            ItersT::Map(f) => (f.0)(&data)?,
            ItersT::Filter(f) => {
                if (f.0)(&data)?.is_included_in(&data::bool::BoolT) {
                    data.clone()
                } else {
                    return Err(format!(
                        "Iter:Filter, but function doesn't return bool for argument {data}."
                    )
                    .into());
                }
            }
            ItersT::FilterMap(f) => {
                if let Some(v) = (f.0)(&data)?.one_tuple_possible_content() {
                    v
                } else {
                    return Err(
                        format!("Iter:FilterMap, but function doesn't return ()/(t).").into(),
                    );
                }
            }
            ItersT::Enumerate => Type::new(data::tuple::TupleT(vec![
                Type::new(data::int::IntT),
                data.clone(),
            ])),
        };
        Ok(Self(iter, data, t))
    }
}
impl MersType for IterT {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        false
    }
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            // TODO: ?
            false
        } else {
            false
        }
    }
    fn iterable(&self) -> Option<Type> {
        Some(self.2.clone())
    }
    fn subtypes(&self, acc: &mut Type) {
        // NOTE: This might not be good enough
        acc.add(Arc::new(self.clone()));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl Display for Iter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Iter>")
    }
}
impl Display for IterT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Iter: {}>", self.2)
    }
}
impl Iters {
    fn as_type(&self) -> ItersT {
        match self {
            Self::Map(f) => ItersT::Map(f.get_as_type()),
            Self::Filter(f) => ItersT::Filter(f.get_as_type()),
            Self::FilterMap(f) => ItersT::FilterMap(f.get_as_type()),
            Self::Enumerate => ItersT::Enumerate,
        }
    }
}
