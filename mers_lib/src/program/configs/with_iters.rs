use std::{fmt::Display, sync::Arc};

use crate::{
    data::{self, Data, MersData},
    program,
};

use super::Config;

impl Config {
    /// Adds functions to deal with iterables
    /// `iter: fn` executes a function once for each element of the iterable
    pub fn with_iters(self) -> Self {
        self.add_var(
            "iter".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
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
                                    "iter called on tuple not containing iterable and function"
                                )
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
            "map".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
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
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
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
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
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
    }
}

#[derive(Clone, Debug)]
pub enum Iters {
    Map(data::function::Function),
    Filter(data::function::Function),
    FilterMap(data::function::Function),
}
#[derive(Clone, Debug)]
pub struct Iter(Iters, Data);
#[derive(Clone, Debug)]
pub struct IterT(Iters);
impl MersData for Iter {
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
                        .filter_map(move |v| f.run(v).get().matches()),
                )
            }
            _ => todo!(),
        })
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
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
