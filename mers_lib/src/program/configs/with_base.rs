use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    program::run::{CheckInfo, Info},
};

use super::Config;

impl Config {
    /// `deref: fn` clones the value from a reference
    /// `eq: fn` returns true if all the values are equal, otherwise false.
    /// `loop: fn` runs a function until it returns (T) instead of (), then returns T.
    /// `len: fn` gets the length of strings or tuples
    pub fn with_base(self) -> Self {
        self.add_var(
            "len".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for t in &a.types {
                        if t.as_any().downcast_ref::<data::string::StringT>().is_none() && t.as_any().downcast_ref::<data::tuple::TupleT>().is_none() {
                            return Err(crate::program::run::CheckError(format!("cannot get length of {t} (must be a tuple or a string)")));
                        }
                    }
                    Ok(Type::new(data::int::IntT))
                }),
                run: Arc::new(|a, _i| {
                    if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        Data::new(data::int::Int(t.0.len() as _))
                    } else if let Some(s) = a.get().as_any().downcast_ref::<data::string::String>() {
                        Data::new(data::int::Int(s.0.len() as _))
                    } else {
                        unreachable!("called len on {a:?}, which isn't a tuple or a string")
                    }
                }),
            }),
        ).add_var(
            "loop".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let mut o = Type::empty();
                    for t in &a.types {
                        if let Some(t) = t.as_any().downcast_ref::<data::function::FunctionT>() {
                            for t in (t.0)(&Type::empty_tuple())?.types {
                                if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                                    if t.0.len() > 1 {
                                        return Err(crate::program::run::CheckError(format!("called loop with funcion that might return a tuple of length > 1")));
                                    } else if let Some(v) = t.0.first() {
                                        o.add(Arc::new(v.clone()))
                                    }
                                } else {
                                    return Err(crate::program::run::CheckError(format!("called loop with funcion that might return something other than a tuple")));
                                }
                            }
                        } else {
                            return Err(crate::program::run::CheckError(format!("called loop on a non-function")));
                        }
                    }
                    Ok(o)
                }),
                run: Arc::new(|a, _i| {
                    if let Some(r) = a.get().as_any().downcast_ref::<data::function::Function>() {
                        loop {
                            if let Some(r) = r.run(Data::empty_tuple()).one_tuple_content() {
                                break r;
                            }
                        }
                    } else {
                        unreachable!("called loop on non-function")
                    }
                }),
            }),
        )
        .add_var(
            "eq".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for t in &a.types {
                            if t.iterable().is_none() {
                                return Err(crate::program::run::CheckError(format!("called eq on non-iterable")))
                            }
                    }
                        Ok(Type::new(data::bool::BoolT))
                    }),
                run: Arc::new(|a, _i| {
                    Data::new(data::bool::Bool(if let Some(mut i) = a.get().iterable() {
                        if let Some(f) = i.next() {
                            let mut o = true;
                            for el in i {
                                if el != f {
                                    o = false;
                                    break;
                                }
                            }
                            o
                        } else {
                            false
                        }
                    } else {
                        false
                    }))
                }),
            }),
        )
        .add_var(
            "deref".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| if let Some(v) = a.dereference() { Ok(v) } else { Err(crate::program::run::CheckError(format!("cannot dereference type {a}")))}),
                run: Arc::new(|a, _i| {
                    if let Some(r) = a
                        .get()
                        .as_any()
                        .downcast_ref::<data::reference::Reference>()
                    {
                        r.0.lock().unwrap().clone()
                    } else {
                        unreachable!("called deref on non-reference")
                    }
                }),
            }),
        )
    }
}
