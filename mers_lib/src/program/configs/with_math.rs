use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, MersType, Type},
    errors::CheckError,
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// `sum: fn` returns the sum of all the numbers in the tuple
    /// `minus: fn` returns the first number minus all the others
    /// `product: fn` returns the product of all the numbers in the tuple
    /// `div: fn` returns a / b. Performs integer division if a and b are both integers.
    /// `modulo: fn` returns a % b
    /// `signum: fn` returns 1 for positive numbers, -1 for negative ones and 0 for 0 (always returns an Int, even when input is Float)
    /// `lt: fn` returns true if the input keeps increasing, that is, for (a, b), a < b, for (a, b, c), a < b < c, and so on.
    /// `gt: fn` returns true if the input keeps decreasing, that is, for (a, b), a > b, for (a, b, c), a > b > c, and so on.
    /// `ltoe: fn` returns true if the input only increases, that is, for (a, b), a <= b, for (a, b, c), a <= b <= c, and so on.
    /// `gtoe: fn` returns true if the input only decreases, that is, for (a, b), a >= b, for (a, b, c), a >= b >= c, and so on.
    /// `parse_int: fn` parses a string to an int, returns () on failure
    /// `parse_float: fn` parses a string to an int, returns () on failure
    /// TODO!
    /// `as_float: fn` turns integers into floats. returns floats without changing them.
    /// `round: fn` rounds the float and returns an int
    /// `ceil: fn` rounds the float [?] and returns an int
    /// `floor: fn` rounds the float [?] and returns an int
    pub fn with_math(self) -> Self {
        self
            .add_var("lt".to_string(), Data::new(ltgtoe_function("lt".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l < r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) < r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l < r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l < r,
            })))
            .add_var("gt".to_string(), Data::new(ltgtoe_function("gt".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l > r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) > r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l > r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l > r,
            })))
            .add_var("ltoe".to_string(), Data::new(ltgtoe_function("ltoe".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l <= r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) <= r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l <= r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l <= r,
            })))
            .add_var("gtoe".to_string(), Data::new(ltgtoe_function("gtoe".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l >= r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) >= r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l >= r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l >= r,
            })))
            .add_var("parse_float".to_string(), Data::new(data::function::Function {
            info: Arc::new(program::run::Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                if a.is_included_in(&Type::new(data::string::StringT)) {
                    Ok(Type::newm(vec![
                        Arc::new(data::float::FloatT),
                        Arc::new(data::tuple::TupleT(vec![])),
                    ]))
                } else {
                    Err(format!("parse_float called on non-string type").into())
                }
            }),
            run: Arc::new(|a, _i| {
                if let Ok(n) = a.get().as_any().downcast_ref::<data::string::String>().unwrap().0.parse() {
                    Data::new(data::float::Float(n))
                } else {
                    Data::empty_tuple()
                }
            }),
                inner_statements: None,
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
                    Err(format!("parse_float called on non-string type").into())
                }
            }),
            run: Arc::new(|a, _i| {
                if let Ok(n) = a.get().as_any().downcast_ref::<data::string::String>().unwrap().0.parse() {
                    Data::new(data::int::Int(n))
                } else {
                    Data::empty_tuple()
                }
            }),
                inner_statements: None,
        })).add_var("signum".to_string(), Data::new(data::function::Function {
            info: Arc::new(program::run::Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                if a.is_included_in(&Type::newm(vec![Arc::new(data::int::IntT), Arc::new(data::float::FloatT)])) {
                    Ok(Type::new(data::int::IntT))
                } else {
                    Err(format!("signum called on non-number type").into())
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
            }),
                inner_statements: None,
        }))            .add_var("div".to_string(), Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| two_tuple_to_num(a, "div")),
                run: Arc::new(|a, _i| if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                    let left = t.0[0].get();
                    let right = t.0[1].get();
                    let (left, right) = (left.as_any(), right.as_any());
                    match (left.downcast_ref::<data::int::Int>(), left.downcast_ref::<data::float::Float>(),
                        right.downcast_ref::<data::int::Int>(), right.downcast_ref::<data::float::Float>()
                    ) {
                        (Some(data::int::Int(l)), None, Some(data::int::Int(r)), None) => Data::new(data::int::Int(l / r)),
                        (Some(data::int::Int(l)), None, None, Some(data::float::Float(r))) => Data::new(data::float::Float(*l as f64 / r)),
                        (None, Some(data::float::Float(l)), Some(data::int::Int(r)), None) => Data::new(data::float::Float(l / *r as f64)),
                        (None, Some(data::float::Float(l)), None, Some(data::float::Float(r))) => Data::new(data::float::Float(l / r)),
                        _ => unreachable!(),
                    }
                } else { unreachable!() }),
                inner_statements: None,
            })).add_var("modulo".to_string(), Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| two_tuple_to_num(a, "modulo")),
                run: Arc::new(|a, _i| if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                    let left = t.0[0].get();
                    let right = t.0[1].get();
                    let (left, right) = (left.as_any(), right.as_any());
                    match (left.downcast_ref::<data::int::Int>(), left.downcast_ref::<data::float::Float>(),
                        right.downcast_ref::<data::int::Int>(), right.downcast_ref::<data::float::Float>()
                    ) {
                        (Some(data::int::Int(l)), None, Some(data::int::Int(r)), None) => Data::new(data::int::Int(l % r)),
                        (Some(data::int::Int(l)), None, None, Some(data::float::Float(r))) => Data::new(data::float::Float(*l as f64 % r)),
                        (None, Some(data::float::Float(l)), Some(data::int::Int(r)), None) => Data::new(data::float::Float(l % *r as f64)),
                        (None, Some(data::float::Float(l)), None, Some(data::float::Float(r))) => Data::new(data::float::Float(l % r)),
                        _ => unreachable!(),
                    }
                } else { unreachable!() }),
                inner_statements: None,
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
                                return Err(format!("cannot get sum of iterator over type {i} because it contains types that aren't int/float").into())
                            }
                        } else {
                            return Err(format!(
                                "cannot get sum of non-iterable type {a}"
                            ).into());
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
                inner_statements: None,
            }),
        )
            .add_var(
            "subtract".to_string(),
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
                                return Err(format!("cannot subtract on iterator over type {i} because it contains types that aren't int/float").into())
                            }
                        } else {
                            return Err(format!(
                                "cannot subtract over non-iterable type {a}"
                            ).into());
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
                        let mut first = true;
                        let mut sumi = 0;
                        let mut sumf = 0.0;
                        let mut usef = false;
                        for val in i {
                            if let Some(i) = val.get().as_any().downcast_ref::<data::int::Int>() {
                                if first {
                                    sumi = i.0;
                                } else {
                                    sumi -= i.0;
                                }
                            } else if let Some(i) =
                                val.get().as_any().downcast_ref::<data::float::Float>()
                            {
                                if first {
                                    sumf = i.0;
                                } else {
                                    sumf -= i.0;
                                }
                                usef = true;
                            }
                            if first {
                                first = false;
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
                inner_statements: None,
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
                                return Err(format!("cannot get product of iterator over type {i} because it contains types that aren't int/float").into())
                            }
                        } else {
                            return Err(format!(
                                "cannot get product of non-iterable type {a}"
                            ).into());
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
                inner_statements: None,
            }),
        )
    }
}

/// (int, int) -> int
/// (int, float) -> float
/// (float, int) -> float
/// (float, float) -> float
fn two_tuple_to_num(a: &Type, func_name: &str) -> Result<Type, CheckError> {
    let mut float = false;
    for t in &a.types {
        if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
            if t.0.len() != 2 {
                return Err(format!("Called {func_name} with a tuple where len != 2").into());
            }
            for (t, side) in [(&t.0[0], "left"), (&t.0[1], "right")] {
                for t in t.types.iter() {
                    if t.as_any().is::<data::float::FloatT>() {
                        float = true;
                    } else if !t.as_any().is::<data::int::IntT>() {
                        return Err(format!("Called {func_name}, but the {side} side of the tuple had type {t}, which isn't Int/Float.").into());
                    }
                }
            }
        } else {
            return Err(format!("Called {func_name} on a non-tuple").into());
        }
    }
    Ok(if a.types.is_empty() {
        Type::empty()
    } else if float {
        Type::new(data::float::FloatT)
    } else {
        Type::new(data::int::IntT)
    })
}

fn ltgtoe_function(
    func_name: String,
    op: impl Fn(IntOrFloatOrNothing, IntOrFloatOrNothing) -> bool + Send + Sync + 'static,
) -> data::function::Function {
    data::function::Function {
        info: Arc::new(program::run::Info::neverused()),
        info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
        out: Arc::new(move |a, _i| {
            if let Some(iter_type) = a.iterable() {
                let iter_required_type = Type::newm(vec![
                    Arc::new(data::int::IntT),
                    Arc::new(data::float::FloatT),
                ]);
                if iter_type.is_included_in(&iter_required_type) {
                    Ok(Type::new(data::bool::BoolT))
                } else {
                    Err(CheckError::from(format!("Cannot use {func_name} on iterator over type {iter_type} (has to be at most {iter_required_type}).")))
                }
            } else {
                Err(CheckError::from(format!("Cannot use {func_name}")))
            }
        }),
        run: Arc::new(move |a, _i| {
            let mut prev = IntOrFloatOrNothing::Nothing;
            for item in a.get().iterable().unwrap() {
                let item = item.get();
                let new = if let Some(data::int::Int(v)) = item.as_any().downcast_ref() {
                    IntOrFloatOrNothing::Int(*v)
                } else if let Some(data::float::Float(v)) = item.as_any().downcast_ref() {
                    IntOrFloatOrNothing::Float(*v)
                } else {
                    unreachable!()
                };
                if op(prev, new) {
                    prev = new;
                } else {
                    return Data::new(data::bool::Bool(false));
                }
            }
            Data::new(data::bool::Bool(true))
        }),
        inner_statements: None,
    }
}
#[derive(Clone, Copy)]
enum IntOrFloatOrNothing {
    Nothing,
    Int(isize),
    Float(f64),
}
