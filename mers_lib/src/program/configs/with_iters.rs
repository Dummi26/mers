use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    data::{
        self,
        function::{Function, FunctionT},
        Data, MersData, MersType, Type,
    },
    errors::CheckError,
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// Adds functions to deal with iterables
    /// `for_each: fn` executes a function once for each element of the iterable
    /// `map: fn` maps each value in the iterable to a new one by applying a transformation function
    /// `filter: fn` filters the iterable by removing all elements where the filter function doesn't return true
    /// `filter_map: fn` combines filter and map. requires that the function returns ()/(t).
    /// `map_while: fn` maps while the map-function returns (d), ends the iterator once () is returned.
    /// `take: fn` takes at most so many elements from the iterator.
    /// `enumerate: fn` transforms an iterator over T into one over (Int, T), where Int is the index of the element
    /// `any: fn` returns true if any element of the iterator are true
    /// `all: fn` returns true if all elements of the iterator are true
    pub fn with_iters(self) -> Self {
        self
            .add_var("any".to_string(), Data::new(genfunc_iter_in_val_out("all".to_string(), data::bool::BoolT, Type::new(data::bool::BoolT), |a, _i| {
                Data::new(data::bool::Bool(a.get().iterable().unwrap().any(|v| v.get().as_any().downcast_ref::<data::bool::Bool>().is_some_and(|v| v.0))))
            })))
            .add_var("all".to_string(), Data::new(genfunc_iter_in_val_out("all".to_string(), data::bool::BoolT, Type::new(data::bool::BoolT), |a, _i| {
                Data::new(data::bool::Bool(a.get().iterable().unwrap().all(|v| v.get().as_any().downcast_ref::<data::bool::Bool>().is_some_and(|v| v.0))))
            })))
            .add_var(
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
                                        let _ret = f.o(&iter)?;
                                        // if !ret.is_zero_tuple() {
                                        //     return Err(format!("for_each function must return (), not {ret}").into());
                                        // }
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
                inner_statements: None,
            }),
        )
        .add_var(
            "map".to_string(),
            Data::new(genfunc_iter_and_func("map", ItersT::Map, Iters::Map))
        )
        .add_var(
            "filter".to_string(),
            Data::new(genfunc_iter_and_func("filter", ItersT::Filter, Iters::Filter)),
        )
        .add_var(
            "filter_map".to_string(),
            Data::new(genfunc_iter_and_func("filter_map", ItersT::FilterMap, Iters::FilterMap)),
        )
        .add_var(
            "map_while".to_string(),
            Data::new(genfunc_iter_and_func("map_while", ItersT::MapWhile, Iters::MapWhile)),
        )
            .add_var("take".to_string(), Data::new(genfunc_iter_and_arg("take", |_: &data::int::IntT| ItersT::Take, |v: &data::int::Int| {
                Iters::Take(v.0.max(0) as _)
            })))
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
                inner_statements: None,
            }),
        )
        .add_var(
            "chain".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let data = if let Some(a) = a.iterable() {
                        a
                    } else {
                        return Err(format!("cannot call chain on non-iterable type {a}.").into());
                    };
                    Ok(Type::new(IterT::new(ItersT::Chained, data)?))
                }),
                run: Arc::new(|a, _i| Data::new(Iter(Iters::Chained, a.clone()))),
                inner_statements: None,
            }),
        )
    }
}

fn iter_out_arg<T: MersType>(
    a: &Type,
    name: &str,
    func: impl Fn(&T) -> ItersT + Sync + Send,
) -> Result<Type, CheckError> {
    let mut out = Type::empty();
    for t in a.types.iter() {
        if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
            if t.0.len() != 2 {
                return Err(format!("cannot call {name} on tuple where len != 2").into());
            }
            if let Some(v) = t.0[0].iterable() {
                for f in t.0[1].types.iter() {
                    if let Some(f) = f.as_any().downcast_ref::<T>() {
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

fn genfunc_iter_and_func(
    name: &'static str,
    ft: impl Fn(FunctionT) -> ItersT + Send + Sync + 'static,
    fd: impl Fn(Function) -> Iters + Send + Sync + 'static,
) -> data::function::Function {
    genfunc_iter_and_arg(
        name,
        move |v| ft(Clone::clone(v)),
        move |v| fd(Clone::clone(v)),
    )
}
fn genfunc_iter_and_arg<T: MersType, D: MersData>(
    name: &'static str,
    ft: impl Fn(&T) -> ItersT + Send + Sync + 'static,
    fd: impl Fn(&D) -> Iters + Send + Sync + 'static,
) -> data::function::Function {
    data::function::Function {
        info: Arc::new(program::run::Info::neverused()),
        info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
        out: Arc::new(move |a, _i| iter_out_arg(a, name, |f: &T| ft(f))),
        run: Arc::new(move |a, _i| {
            if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                    if let Some(f) = f.get().as_any().downcast_ref::<D>() {
                        Data::new(Iter(fd(f), v.clone()))
                    } else {
                        unreachable!("{name} called on tuple not containing function")
                    }
                } else {
                    unreachable!("{name} called on tuple with len < 2")
                }
            } else {
                unreachable!("{name} called on non-tuple")
            }
        }),
        inner_statements: None,
    }
}

#[derive(Clone, Debug)]
pub enum Iters {
    Map(data::function::Function),
    Filter(data::function::Function),
    FilterMap(data::function::Function),
    MapWhile(data::function::Function),
    Take(usize),
    Enumerate,
    Chained,
}
#[derive(Clone, Debug)]
pub enum ItersT {
    Map(data::function::FunctionT),
    Filter(data::function::FunctionT),
    FilterMap(data::function::FunctionT),
    MapWhile(data::function::FunctionT),
    Take,
    Enumerate,
    Chained,
}
#[derive(Clone, Debug)]
pub struct Iter(pub Iters, pub Data);
#[derive(Clone, Debug)]
pub struct IterT(pub ItersT, pub Type, pub Type);
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
            Iters::MapWhile(f) => {
                let f = Clone::clone(f);
                Box::new(
                    self.1
                        .get()
                        .iterable()?
                        .map_while(move |v| f.run(v).one_tuple_content()),
                )
            }
            Iters::Take(limit) => Box::new(self.1.get().iterable()?.take(*limit)),
            Iters::Enumerate => Box::new(self.1.get().iterable()?.enumerate().map(|(i, v)| {
                Data::new(data::tuple::Tuple(vec![
                    Data::new(data::int::Int(i as _)),
                    v,
                ]))
            })),
            Iters::Chained => {
                let iters = self
                    .1
                    .get()
                    .iterable()?
                    .map(|v| v.get().iterable())
                    .collect::<Option<Vec<_>>>()?;
                Box::new(iters.into_iter().flatten())
            }
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
            ItersT::Map(f) => f.o(&data)?,
            ItersT::Filter(f) => {
                if f.o(&data)?.is_included_in_single(&data::bool::BoolT) {
                    data.clone()
                } else {
                    return Err(format!(
                        "Iter:Filter, but function doesn't return bool for argument {data}."
                    )
                    .into());
                }
            }
            ItersT::FilterMap(f) => {
                if let Some(v) = f.o(&data)?.one_tuple_possible_content() {
                    v
                } else {
                    return Err(
                        format!("Iter:FilterMap, but function doesn't return ()/(t).").into(),
                    );
                }
            }
            ItersT::MapWhile(f) => {
                if let Some(t) = f.o(&data)?.one_tuple_possible_content() {
                    t
                } else {
                    return Err(
                        format!("Iter:MapWhile, but function doesn't return ()/(t).").into(),
                    );
                }
            }
            ItersT::Take => data.clone(),
            ItersT::Enumerate => Type::new(data::tuple::TupleT(vec![
                Type::new(data::int::IntT),
                data.clone(),
            ])),
            ItersT::Chained => {
                if let Some(out) = data.iterable() {
                    out
                } else {
                    return Err(format!(
                        "Cannot create a chain from an iterator over the non-iterator type {data}."
                    )
                    .into());
                }
            }
        };
        Ok(Self(iter, data, t))
    }
}
impl MersType for IterT {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.2.is_same_type_as(&other.2)
        } else {
            false
        }
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            self.2.is_included_in(&target.2)
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
            Self::MapWhile(f) => ItersT::MapWhile(f.get_as_type()),
            Self::Take(_) => ItersT::Take,
            Self::Enumerate => ItersT::Enumerate,
            Self::Chained => ItersT::Chained,
        }
    }
}

fn genfunc_iter_in_val_out(
    name: String,
    iter_type: impl MersType + 'static,
    out_type: Type,
    run: impl Fn(Data, &mut crate::info::Info<program::run::Local>) -> Data + Send + Sync + 'static,
) -> Function {
    Function {
        info: Arc::new(crate::info::Info::neverused()),
        info_check: Arc::new(Mutex::new(crate::info::Info::neverused())),
        out: Arc::new(move |a, _i| {
            if let Some(iter_over) = a.iterable() {
                if iter_over.is_included_in_single(&iter_type) {
                    Ok(out_type.clone())
                } else {
                    Err(format!("Cannot call function {name} on iterator over type {a}, which isn't {iter_type}.").into())
                }
            } else {
                Err(format!("Cannot call function {name} on non-iterable type {a}.").into())
            }
        }),
        run: Arc::new(run),
        inner_statements: None,
    }
}
