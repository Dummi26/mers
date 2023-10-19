use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, MersType, Type},
    program::{
        self,
        run::{CheckError, CheckInfo},
    },
};

use super::Config;

impl Config {
    /// `sum: fn` returns the sum of all the numbers in the tuple
    /// `diff: fn` returns b - a
    /// `product: fn` returns the product of all the numbers in the tuple
    /// `signum: fn` returns 1 for positive numbers, -1 for negative ones and 0 for 0 (always returns an Int, even when input is Float)
    /// `parse_int: fn` parses a string to an int, returns () on failure
    /// `parse_float: fn` parses a string to an int, returns () on failure
    /// TODO!
    /// `as_float: fn` turns integers into floats. returns floats without changing them.
    /// `round: fn` rounds the float and returns an int
    /// `ceil: fn` rounds the float [?] and returns an int
    /// `floor: fn` rounds the float [?] and returns an int
    /// `div: fn` returns a / b. Performs integer division if a and b are both integers.
    /// `modulo: fn` returns a % b
    pub fn with_math(self) -> Self {
        self.add_var("parse_float".to_string(), Data::new(data::function::Function {
            info: Arc::new(program::run::Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                if a.is_included_in(&Type::new(data::string::StringT)) {
                    Ok(Type::newm(vec![
                        Arc::new(data::float::FloatT),
                        Arc::new(data::tuple::TupleT(vec![])),
                    ]))
                } else {
                    Err(CheckError(format!("parse_float called on non-string type")))
                }
            }),
            run: Arc::new(|a, _i| {
                if let Ok(n) = a.get().as_any().downcast_ref::<data::string::String>().unwrap().0.parse() {
                    Data::new(data::float::Float(n))
                } else {
                    Data::empty_tuple()
                }
            })
        })).add_var("parse_int".to_string(), Data::new(data::function::Function {
            info: Arc::new(program::run::Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                if a.is_included_in(&Type::new(data::string::StringT)) {
                    Ok(Type::newm(vec![
                        Arc::new(data::int::IntT),
                        Arc::new(data::tuple::TupleT(vec![])),
                    ]))
                } else {
                    Err(CheckError(format!("parse_float called on non-string type")))
                }
            }),
            run: Arc::new(|a, _i| {
                if let Ok(n) = a.get().as_any().downcast_ref::<data::string::String>().unwrap().0.parse() {
                    Data::new(data::int::Int(n))
                } else {
                    Data::empty_tuple()
                }
            })
        })).add_var("signum".to_string(), Data::new(data::function::Function {
            info: Arc::new(program::run::Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                if a.is_included_in(&Type::newm(vec![Arc::new(data::int::IntT), Arc::new(data::float::FloatT)])) {
                    Ok(Type::new(data::int::IntT))
                } else {
                    Err(CheckError(format!("signum called on non-number type")))
                }
            }),
            run: Arc::new(|a, _i| {
                Data::new(data::int::Int(if let Some(n) = a.get().as_any().downcast_ref::<data::int::Int>() {
                    n.0.signum()
                } else
                if let Some(n) = a.get().as_any().downcast_ref::<data::float::Float>() {
                    if n.0 > 0.0 {
                        1
                    } else if n.0 < 0.0 {
                        -1
                    } else { 0
                    }
                } else { unreachable!("called signum on non-number type")}))
            })
        }))
            .add_var("diff".to_string(), Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let mut float = false;
                    for t in &a.types {
                        if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                            if t.0.len() != 2 {
                                return Err(CheckError(format!("Called diff with a tuple where len != 2")));
                            }
                            for (t, side) in [(&t.0[0], "left"), (&t.0[1], "right")] {
                                for t in t.types.iter() {
                                    if t.as_any().is::<data::float::FloatT>() {
                                        float = true;
                                    } else if !t.as_any().is::<data::int::IntT>() {
                                        return Err(CheckError(format!("Called diff, but the {side} side of the tuple had type {t}, which isn't Int/Float.")));
                                    }
                                }
                            }
                        } else {
                            return Err(CheckError(format!("Called diff on a non-tuple")));
                        }
                    }
                    Ok(if a.types.is_empty() {
                        Type::empty()
                    } else if float {
                        Type::new(data::float::FloatT)
                    } else {
                        Type::new(data::int::IntT)
                    })
                }),
                run: Arc::new(|a, _i| if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                    let left = t.0[0].get();
                    let right = t.0[1].get();
                    let (left, right) = (left.as_any(), right.as_any());
                    match (left.downcast_ref::<data::int::Int>(), left.downcast_ref::<data::float::Float>(),
                        right.downcast_ref::<data::int::Int>(), right.downcast_ref::<data::float::Float>()
                    ) {
                        (Some(data::int::Int(l)), None, Some(data::int::Int(r)), None) => Data::new(data::int::Int(r - l)),
                        (Some(data::int::Int(l)), None, None, Some(data::float::Float(r))) => Data::new(data::float::Float(r - *l as f64)),
                        (None, Some(data::float::Float(l)), Some(data::int::Int(r)), None) => Data::new(data::float::Float(*r as f64 - l)),
                        (None, Some(data::float::Float(l)), None, Some(data::float::Float(r))) => Data::new(data::float::Float(r - l)),
                        _ => unreachable!(),
                    }
                } else { unreachable!() }),
            }))
            .add_var(
            "sum".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let mut ints = false;
                    let mut floats = false;
                    for a in &a.types {
                        if let Some(i) = a.iterable() {
                            if i.types
                                .iter()
                                .all(|t| t.as_any().downcast_ref::<data::int::IntT>().is_some())
                            {
                                ints = true;
                            } else if i.types.iter().all(|t| {
                                t.as_any().downcast_ref::<data::int::IntT>().is_some()
                                    || t.as_any().downcast_ref::<data::float::FloatT>().is_some()
                            }) {
                                floats = true;
                            } else {
                                return Err(CheckError(format!("cannot get sum of iterator over type {i} because it contains types that aren't int/float")))
                            }
                        } else {
                            return Err(CheckError(format!(
                                "cannot get sum of non-iterable type {a}"
                            )));
                        }
                    }
                    Ok(match (ints, floats) {
                        (_, true) => Type::new(data::float::FloatT),
                        (true, false) => Type::new(data::int::IntT),
                        (false, false) => Type::empty(),
                    })
                }),
                run: Arc::new(|a, _i| {
                    if let Some(i) = a.get().iterable() {
                        let mut sumi = 0;
                        let mut sumf = 0.0;
                        let mut usef = false;
                        for val in i {
                            if let Some(i) = val.get().as_any().downcast_ref::<data::int::Int>() {
                                sumi += i.0;
                            } else if let Some(i) =
                                val.get().as_any().downcast_ref::<data::float::Float>()
                            {
                                sumf += i.0;
                                usef = true;
                            }
                        }
                        if usef {
                            Data::new(data::float::Float(sumi as f64 + sumf))
                        } else {
                            Data::new(data::int::Int(sumi))
                        }
                    } else {
                        unreachable!("sum called on non-tuple")
                    }
                }),
            }),
        )
            .add_var(
            "product".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let mut ints = false;
                    let mut floats = false;
                    for a in &a.types {
                        if let Some(i) = a.iterable() {
                            if i.types
                                .iter()
                                .all(|t| t.as_any().downcast_ref::<data::int::IntT>().is_some())
                            {
                                ints = true;
                            } else if i.types.iter().all(|t| {
                                t.as_any().downcast_ref::<data::int::IntT>().is_some()
                                    || t.as_any().downcast_ref::<data::float::FloatT>().is_some()
                            }) {
                                floats = true;
                            } else {
                                return Err(CheckError(format!("cannot get product of iterator over type {i} because it contains types that aren't int/float")))
                            }
                        } else {
                            return Err(CheckError(format!(
                                "cannot get product of non-iterable type {a}"
                            )));
                        }
                    }
                    Ok(match (ints, floats) {
                        (_, true) => Type::new(data::float::FloatT),
                        (true, false) => Type::new(data::int::IntT),
                        (false, false) => Type::empty(),
                    })
                }),
                run: Arc::new(|a, _i| {
                    if let Some(i) = a.get().iterable() {
                        let mut prodi = 1;
                        let mut prodf = 1.0;
                        let mut usef = false;
                        for val in i {
                            if let Some(i) = val.get().as_any().downcast_ref::<data::int::Int>() {
                                prodi *= i.0;
                            } else if let Some(i) =
                                val.get().as_any().downcast_ref::<data::float::Float>()
                            {
                                prodf *= i.0;
                                usef = true;
                            }
                        }
                        if usef {
                            Data::new(data::float::Float(prodi as f64 * prodf))
                        } else {
                            Data::new(data::int::Int(prodi))
                        }
                    } else {
                        unreachable!("product called on non-tuple")
                    }
                }),
            }),
        )
    }
}
