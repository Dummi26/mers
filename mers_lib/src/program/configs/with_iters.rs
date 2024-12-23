use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    data::{
        self,
        function::{Function, FunctionT},
        int::{Int, IntT, INT_MAX},
        Data, MersData, MersType, MersTypeWInfo, Type,
    },
    errors::CheckError,
    info::DisplayInfo,
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
    /// `range_inc: fn` returns an iterable `Range` starting at the first argument, counting up to the second one (inclusive).
    /// `range_exc: fn` returns an iterable `Range` starting at the first argument, counting up to the second one (exclusive).
    pub fn with_iters(self) -> Self {
        self
            .add_type("Range".to_owned(), Err(Arc::new(|str, _i| {
                if let Some((val, end)) = str.split_once("..") {
                    if let (Ok(val), Ok(end)) = (val.trim().parse(), end.trim().parse()) {
                        Ok(RangeT(val, end))
                    } else {
                        Err(CheckError::from(format!("bad Range type, got <{str}> but expected <start..end> where start and end are Ints")))
                    }
                } else {
                    Err(CheckError::from(format!("bad Range type, got <{str}> but expected <start..end>")))
                }.map(|v| Arc::new(Type::new(v)))
            })))
            .add_var("range_inc", Function::new_generic(
                |a, i| {
                    let mut o = Type::empty();
                    for a in &a.types {
                        let a = a.as_any().downcast_ref::<data::tuple::TupleT>().ok_or_else(|| CheckError::from(format!("expected 2- or 3-tuple, but found {}", a.with_info(i))))?;
                        if a.0.len() == 2 {
                            let mut min = None;
                            let mut max = None;
                            for v in &a.0[0].types {
                                let v = v.as_any().downcast_ref::<data::int::IntT>().ok_or_else(|| CheckError::from(format!("expected int as first argument, but got {}", v.with_info(i))))?;
                                if min.is_none_or(|min| min > v.0) {
                                    min = Some(v.0);
                                }
                            }
                            for v in &a.0[1].types {
                                let v = v.as_any().downcast_ref::<data::int::IntT>().ok_or_else(|| CheckError::from(format!("expected int as second argument, but got {}", v.with_info(i))))?;
                                if max.is_none_or(|max| max < v.1) {
                                    max = Some(v.1);
                                }
                            }
                            if let (Some(min), Some(max)) = (min, max) {
                                o.add(Arc::new(RangeT(min, max)));
                            }
                        } else {
                            return Err(CheckError::from(format!("expected 2-tuple, but found {}", a.with_info(i))));
                        }
                    }
                    Ok(o)
                }, |a, _| {
                    let a = a.get();
                    let a = a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                    let (v, e) = (a.0[0].get(), a.0[1].get());
                    let (v, e) = (v.as_any().downcast_ref::<data::int::Int>().unwrap(), e.as_any().downcast_ref::<data::int::Int>().unwrap());
                    Ok(Data::new(Range(v.0, e.0)))
                }
            ))
            .add_var("range_exc", Function::new_generic(
                |a, i| {
                    let mut o = Type::empty();
                    for a in &a.types {
                        let a = a.as_any().downcast_ref::<data::tuple::TupleT>().ok_or_else(|| CheckError::from(format!("expected 2- or 3-tuple, but found {}", a.with_info(i))))?;
                        if a.0.len() == 2 {
                            let mut min = None;
                            let mut max = None;
                            for v in &a.0[0].types {
                                let v = v.as_any().downcast_ref::<data::int::IntT>().ok_or_else(|| CheckError::from(format!("expected int as first argument, but got {}", v.with_info(i))))?;
                                if min.is_none_or(|min| min > v.0) {
                                    min = Some(v.0);
                                }
                            }
                            for v in &a.0[1].types {
                                let v = v.as_any().downcast_ref::<data::int::IntT>().ok_or_else(|| CheckError::from(format!("expected int as second argument, but got {}", v.with_info(i))))?;
                                if max.is_none_or(|max| max < v.1) {
                                    max = Some(v.1);
                                }
                            }
                            if let (Some(min), Some(max)) = (min, max) {
                                if let Some(max) = max.checked_sub(1) {
                                    o.add(Arc::new(RangeT(min, max)));
                                } else {
                                    o.add(Arc::new(RangeT(if min == isize::MIN { min + 1 } else { min }, isize::MIN)));
                                }
                            }
                        } else {
                            return Err(CheckError::from(format!("expected 2-tuple, but found {}", a.with_info(i))));
                        }
                    }
                    Ok(o)
                }, |a, _| {
                    let a = a.get();
                    let a = a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                    let (v, e) = (a.0[0].get(), a.0[1].get());
                    let (v, e) = (v.as_any().downcast_ref::<data::int::Int>().unwrap(), e.as_any().downcast_ref::<data::int::Int>().unwrap());
                    if let Some(e) = e.0.checked_sub(1) {
                        Ok(Data::new(Range(v.0, e)))
                    } else {
                        Ok(Data::new(Range(v.0.saturating_add(1), e.0)))
                    }
                }
            ))
            .add_var("any", genfunc_iter_in_val_out("all".to_string(), data::bool::bool_type(), data::bool::bool_type(), |a, i| {
                for v in a.get().iterable(&i.global).unwrap().map(|v| v.map(|v| v.get().as_any().downcast_ref::<data::bool::Bool>().is_some_and(|v| v.0))) {
                    if v? {
                        return Ok(Data::new(data::bool::Bool(true)));
                    }
                }
                Ok(Data::new(data::bool::Bool(false)))
            }))
            .add_var("all", genfunc_iter_in_val_out("all".to_string(), data::bool::bool_type(), data::bool::bool_type(), |a, i| {
                for v in a.get().iterable(&i.global).unwrap().map(|v| v.map(|v| v.get().as_any().downcast_ref::<data::bool::Bool>().is_some_and(|v| v.0))) {
                    if !v? {
                        return Ok(Data::new(data::bool::Bool(false)));
                    }
                }
                Ok(Data::new(data::bool::Bool(true)))
            }))
            .add_var(
            "for_each",
            data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, i| {
                    for a in &a.types {
                        if let Some(tuple) = a.as_any().downcast_ref::<data::tuple::TupleT>() {
                            if let (Some(v), Some(f)) = (tuple.0.get(0), tuple.0.get(1)) {
                                if let (Some(iter), Some(f)) = (
                                    v.iterable(),
                                    f.types
                                        .iter()
                                        .map(|t| {
                                            t.executable()
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
                                        "for_each called on tuple not containing iterable and function: {} is {}",
                                        v.with_info(i),
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
                })),
                run: Arc::new(|a, i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                            if let Some(iter) = v.get().iterable(&i.global) {
                                let f = f.get();
                                for v in iter {
                                    f.execute(v?, &i.global).unwrap()?;
                                }
                                Ok(Data::empty_tuple())
                            } else {
                                return Err(
                                    "for_each called on tuple not containing iterable and function (not an iterable)".into()
                                );
                            }
                        } else {
                            return Err("for_each called on tuple with len < 2".into());
                        }
                    } else {
                        return Err("for_each called on non-tuple".into());
                    }
                }),
                inner_statements: None,
            },
        )
        .add_var(
            "map",
            genfunc_iter_and_func("map", ItersT::Map, Iters::Map)
        )
        .add_var(
            "filter",
            genfunc_iter_and_func("filter", ItersT::Filter, Iters::Filter),
        )
        .add_var(
            "filter_map",
            genfunc_iter_and_func("filter_map", ItersT::FilterMap, Iters::FilterMap),
        )
        .add_var(
            "map_while",
            genfunc_iter_and_func("map_while", ItersT::MapWhile, Iters::MapWhile),
        )
        .add_var("take", genfunc_iter_and_arg("take", |_: &data::int::IntT| ItersT::Take, |v: &data::int::Int| {
            Iters::Take(v.0.max(0).try_into().unwrap_or(usize::MAX))
        }, &data::int::IntT(0, INT_MAX)))
        .add_var(
            "enumerate",
            data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, i| {
                    let data = if let Some(a) = a.iterable() {
                        a
                    } else {
                        return Err(format!("cannot call enumerate on non-iterable type {}.", a.with_info(i)).into());
                    };
                    Ok(Type::new(IterT::new(ItersT::Enumerate, data, i)?))
                })),
                run: Arc::new(|a, _i| Ok(Data::new(Iter(Iters::Enumerate, a.clone())))),
                inner_statements: None,
            },
        )
        .add_var(
            "chain",
            data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, i| {
                    let data = if let Some(a) = a.iterable() {
                        a
                    } else {
                        return Err(format!("cannot call chain on non-iterable type {}.", a.with_info(i)).into());
                    };
                    Ok(Type::new(IterT::new(ItersT::Chained, data, i)?))
                })),
                run: Arc::new(|a, _i| Ok(Data::new(Iter(Iters::Chained, a.clone())))),
                inner_statements: None,
            },
        )
    }
}

fn genfunc_iter_and_func(
    name: &'static str,
    ft: impl Fn(FunctionT) -> ItersT + Send + Sync + 'static,
    fd: impl Fn(Data) -> Iters + Send + Sync + 'static,
) -> data::function::Function {
    fn iter_out_arg(
        a: &Type,
        i: &mut CheckInfo,
        name: &str,
        func: impl Fn(FunctionT) -> ItersT + Sync + Send,
    ) -> Result<Type, CheckError> {
        let mut out = Type::empty();
        for t in a.types.iter() {
            if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                if t.0.len() != 2 {
                    return Err(format!("cannot call {name} on tuple where len != 2").into());
                }
                if let Some(v) = t.0[0].iterable() {
                    for f in t.0[1].types.iter() {
                        if let Some(f) = f.executable() {
                            out.add(Arc::new(IterT::new(func(f), v.clone(), i)?));
                        } else {
                            return Err(format!("cannot call {name} on tuple that isn't (_, function): got {} instead of function as part of {}", t.0[1].with_info(i), a.with_info(i)).into());
                        }
                    }
                } else {
                    return Err(format!(
                        "cannot call {name} on non-iterable type {}, which is part of {}.",
                        t.with_info(i),
                        a.with_info(i)
                    )
                    .into());
                }
            }
        }
        Ok(out)
    }
    data::function::Function {
        info: program::run::Info::neverused(),
        info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
        out: Ok(Arc::new(move |a, i| iter_out_arg(a, i, name, |f| ft(f)))),
        run: Arc::new(move |a, _i| {
            if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                    Ok(Data::new(Iter(fd(f.clone()), v.clone())))
                } else {
                    return Err("{name} called on tuple with len < 2".into());
                }
            } else {
                return Err("{name} called on non-tuple".into());
            }
        }),
        inner_statements: None,
    }
}
fn genfunc_iter_and_arg<T: MersType, D: MersData>(
    name: &'static str,
    ft: impl Fn(&T) -> ItersT + Send + Sync + 'static,
    fd: impl Fn(&D) -> Iters + Send + Sync + 'static,
    type_sample: &'static T,
) -> data::function::Function {
    fn iter_out_arg<T: MersType>(
        a: &Type,
        i: &mut CheckInfo,
        name: &str,
        func: impl Fn(&T) -> ItersT + Sync + Send,
        type_sample: &T,
    ) -> Result<Type, CheckError> {
        let type_sample = type_sample.with_info(i);
        let mut out = Type::empty();
        for t in a.types.iter() {
            if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                if t.0.len() != 2 {
                    return Err(format!("cannot call {name} on tuple where len != 2").into());
                }
                if let Some(v) = t.0[0].iterable() {
                    for f in t.0[1].types.iter() {
                        if let Some(f) = f.as_any().downcast_ref::<T>() {
                            out.add(Arc::new(IterT::new(func(f), v.clone(), i)?));
                        } else {
                            return Err(format!("cannot call {name} on tuple that isn't (_, {type_sample}): got {} instead of {type_sample} as part of {}", t.0[1].with_info(i), a.with_info(i)).into());
                        }
                    }
                } else {
                    return Err(format!(
                        "cannot call {name} on non-iterable type {}, which is part of {}.",
                        t.with_info(i),
                        a.with_info(i)
                    )
                    .into());
                }
            }
        }
        Ok(out)
    }
    data::function::Function {
        info: program::run::Info::neverused(),
        info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
        out: Ok(Arc::new(move |a, i| {
            iter_out_arg(a, i, name, |f: &T| ft(f), type_sample)
        })),
        run: Arc::new(move |a, _i| {
            if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                if let (Some(v), Some(f)) = (tuple.get(0), tuple.get(1)) {
                    if let Some(f) = f.get().as_any().downcast_ref::<D>() {
                        Ok(Data::new(Iter(fd(f), v.clone())))
                    } else {
                        return Err("{name} called on tuple not containing function".into());
                    }
                } else {
                    return Err("{name} called on tuple with len < 2".into());
                }
            } else {
                return Err("{name} called on non-tuple".into());
            }
        }),
        inner_statements: None,
    }
}

#[derive(Clone, Debug)]
pub enum Iters {
    Map(Data),
    Filter(Data),
    FilterMap(Data),
    MapWhile(Data),
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
    fn display(&self, _info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
    fn is_eq(&self, _other: &dyn MersData) -> bool {
        false
    }
    fn iterable(
        &self,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        let gi = gi.clone();
        Some(match &self.0 {
            Iters::Map(f) => {
                let f = Clone::clone(f);
                Box::new(self.1.get().iterable(&gi)?.map(move |v| {
                    f.get()
                        .execute(v?, &gi)
                        .ok_or_else(|| CheckError::from("called map with non-function argument"))?
                }))
            }
            Iters::Filter(f) => {
                let f = Clone::clone(f);
                Box::new(self.1.get().iterable(&gi)?.filter_map(move |v| {
                    match v {
                        Ok(v) => match f.get().execute(v.clone(), &gi) {
                            Some(Ok(f)) => {
                                if f.get()
                                    .as_any()
                                    .downcast_ref::<data::bool::Bool>()
                                    .is_some_and(|b| b.0)
                                {
                                    Some(Ok(v))
                                } else {
                                    None
                                }
                            }
                            Some(Err(e)) => Some(Err(e)),
                            None => Some(Err(CheckError::from(
                                "called filter with non-function argument",
                            ))),
                        },
                        Err(e) => Some(Err(e)),
                    }
                }))
            }
            Iters::FilterMap(f) => {
                let f = Clone::clone(f);
                Box::new(self.1.get().iterable(&gi)?.filter_map(move |v| match v {
                    Ok(v) => match f.get().execute(v, &gi) {
                        Some(Ok(r)) => Some(Ok(r.one_tuple_content()?)),
                        Some(Err(e)) => Some(Err(e)),
                        None => Some(Err(CheckError::from(
                            "called filter_map with non-function argument",
                        ))),
                    },
                    Err(e) => Some(Err(e)),
                }))
            }
            Iters::MapWhile(f) => {
                let f = Clone::clone(f);
                Box::new(self.1.get().iterable(&gi)?.map_while(move |v| match v {
                    Ok(v) => match f.get().execute(v, &gi) {
                        Some(Ok(r)) => Some(Ok(r.one_tuple_content()?)),
                        Some(Err(e)) => Some(Err(e)),
                        None => Some(Err(CheckError::from(
                            "called map_while with non-function argument",
                        ))),
                    },
                    Err(e) => Some(Err(e)),
                }))
            }
            Iters::Take(limit) => Box::new(self.1.get().iterable(&gi)?.take(*limit)),
            Iters::Enumerate => {
                Box::new(
                    self.1
                        .get()
                        .iterable(&gi)?
                        .enumerate()
                        .map(|(i, v)| match v {
                            Ok(v) => Ok(Data::new(data::tuple::Tuple(vec![
                                Data::new(data::int::Int(i as _)),
                                v,
                            ]))),
                            Err(e) => Err(e),
                        }),
                )
            }
            Iters::Chained => {
                match self
                    .1
                    .get()
                    .iterable(&gi)?
                    .map(move |v| Ok(v?.get().iterable(&gi)))
                    .collect::<Result<Option<Vec<_>>, CheckError>>()
                {
                    Ok(Some(iters)) => Box::new(iters.into_iter().flatten()),
                    Ok(None) => return None,
                    Err(e) => Box::new([Err(e)].into_iter()),
                }
            }
        })
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> data::Type {
        Type::new(
            IterT::new(
                self.0.as_type(),
                self.1.get().as_type(),
                &crate::info::Info::neverused(),
            )
            .unwrap(),
        )
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
    /// `i` is only used for errors (this is important for `as_type()`)
    pub fn new(iter: ItersT, data: Type, i: &CheckInfo) -> Result<Self, CheckError> {
        let t = match &iter {
            ItersT::Map(f) => f.o(&data)?,
            ItersT::Filter(f) => {
                if f.o(&data)?.is_included_in(&data::bool::bool_type()) {
                    data.clone()
                } else {
                    return Err(format!(
                        "Iter:Filter, but function doesn't return bool for argument {}.",
                        data.with_info(&i)
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
                Type::new(data::int::IntT(0, INT_MAX)),
                data.clone(),
            ])),
            ItersT::Chained => {
                if let Some(out) = data.iterable() {
                    out
                } else {
                    return Err(format!(
                        "Cannot create a chain from an iterator over the non-iterator type {}.",
                        data.with_info(i)
                    )
                    .into());
                }
            }
        };
        Ok(Self(iter, data, t))
    }
}
impl MersType for IterT {
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "<Iter: {}>", self.2.with_display(info))
    }
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
impl Iters {
    fn as_type(&self) -> ItersT {
        match self {
            Self::Map(f) => ItersT::Map(f.get().executable().unwrap()),
            Self::Filter(f) => ItersT::Filter(f.get().executable().unwrap()),
            Self::FilterMap(f) => ItersT::FilterMap(f.get().executable().unwrap()),
            Self::MapWhile(f) => ItersT::MapWhile(f.get().executable().unwrap()),
            Self::Take(_) => ItersT::Take,
            Self::Enumerate => ItersT::Enumerate,
            Self::Chained => ItersT::Chained,
        }
    }
}

fn genfunc_iter_in_val_out(
    name: String,
    iter_type: Type,
    out_type: Type,
    run: impl Fn(Data, &mut crate::info::Info<program::run::RunLocal>) -> Result<Data, CheckError>
        + Send
        + Sync
        + 'static,
) -> Function {
    Function {
        info: crate::info::Info::neverused(),
        info_check: Arc::new(Mutex::new(crate::info::Info::neverused())),
        out: Ok(Arc::new(move |a, i| {
            if let Some(iter_over) = a.iterable() {
                if iter_over.is_included_in(&iter_type) {
                    Ok(out_type.clone())
                } else {
                    Err(format!(
                        "Cannot call function {name} on iterator over type {}, which isn't {}.",
                        a.with_info(i),
                        iter_type.with_info(i)
                    )
                    .into())
                }
            } else {
                Err(format!(
                    "Cannot call function {name} on non-iterable type {}.",
                    a.with_info(i)
                )
                .into())
            }
        })),
        run: Arc::new(run),
        inner_statements: None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Range(isize, isize);
#[derive(Clone, Debug, PartialEq)]
pub struct RangeT(isize, isize);
impl MersData for Range {
    fn display(&self, _info: &DisplayInfo, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.range_inc({})", self.0, self.1)
    }
    fn iterable(
        &self,
        _gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        Some(Box::new(
            RangeInt(self.0, self.1, false).map(|v| Ok(Data::new(Int(v)))),
        ))
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn is_eq(&self, other: &dyn MersData) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| other == self)
    }
    fn as_type(&self) -> Type {
        Type::new(RangeT(self.0, self.1))
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
impl MersType for RangeT {
    fn display(
        &self,
        _info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "Range<{}..{}>", self.0, self.1)
    }
    fn iterable(&self) -> Option<Type> {
        Some(if self.is_empty() {
            Type::empty()
        } else {
            Type::new(IntT(self.0, self.1))
        })
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| *self == *other)
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        target
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|target| {
                // prolly good
                self.is_empty() || (!target.is_empty() && self.0 >= target.0 && self.1 <= target.1)
            })
    }
    fn subtypes(&self, acc: &mut Type) {
        acc.add(Arc::new(Clone::clone(self)))
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
impl RangeT {
    pub fn is_empty(&self) -> bool {
        self.1 < self.0
    }
}

struct RangeInt(isize, isize, bool);
impl Iterator for RangeInt {
    type Item = isize;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.2 && self.0 <= self.1 {
            let o = self.0;
            if self.0 < self.1 {
                self.0 += 1;
            } else {
                self.2 = true;
            }
            Some(o)
        } else {
            None
        }
    }
}
