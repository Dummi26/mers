use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    data::{self, Data, MersData, MersType, Type},
    program::{self, run::CheckInfo},
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
                out: Arc::new(|a, i| {
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
                out: Arc::new(|a, i| todo!()),
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
                out: Arc::new(|a, i| todo!()),
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
                out: Arc::new(|a, i| todo!()),
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
                out: Arc::new(|a, i| todo!()),
                run: Arc::new(|a, _i| Data::new(Iter(Iters::Enumerate, a.clone()))),
            }),
        )
    }
}

#[derive(Clone, Debug)]
pub enum Iters {
    Map(data::function::Function),
    Filter(data::function::Function),
    FilterMap(data::function::Function),
    Enumerate,
}
#[derive(Clone, Debug)]
pub struct Iter(Iters, Data);
#[derive(Clone, Debug)]
pub struct IterT(Iters);
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
        Type::new(IterT(self.0.clone()))
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
        write!(f, "<Iter>")
    }
}
