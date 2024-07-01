use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// `sum: fn` returns the sum of all the numbers in the tuple
    /// `minus: fn` returns the first number minus all the others
    /// `product: fn` returns the product of all the numbers in the tuple
    /// `div: fn` returns a / b. Performs integer division if a and b are both integers.
    /// `pow: fn` returns a^b or a**b.
    /// `modulo: fn` returns a % b
    /// `signum: fn` returns 1 for positive numbers, -1 for negative ones and 0 for 0 (always returns an Int, even when input is Float)
    /// `lt: fn` returns true if the input keeps increasing, that is, for (a, b), a < b, for (a, b, c), a < b < c, and so on.
    /// `gt: fn` returns true if the input keeps decreasing, that is, for (a, b), a > b, for (a, b, c), a > b > c, and so on.
    /// `ltoe: fn` returns true if the input only increases, that is, for (a, b), a <= b, for (a, b, c), a <= b <= c, and so on.
    /// `gtoe: fn` returns true if the input only decreases, that is, for (a, b), a >= b, for (a, b, c), a >= b >= c, and so on.
    /// `parse_int: fn` parses a string to an int, returns () on failure and (Int) otherwise
    /// `parse_float: fn` parses a string to an int, returns () on failure and (Int) otherwise
    /// TODO!
    /// `as_float: fn` turns integers into floats. returns floats without changing them.
    /// `round: fn` rounds the float and returns an int
    /// `ceil: fn` rounds the float [?] and returns an int
    /// `floor: fn` rounds the float [?] and returns an int
    pub fn with_math(self) -> Self {
        self.add_var(
            "lt".to_string(),
            Data::new(ltgtoe_function("lt".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l < r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) < r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l < r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l < r,
            })),
        )
        .add_var(
            "gt".to_string(),
            Data::new(ltgtoe_function("gt".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l > r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) > r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l > r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l > r,
            })),
        )
        .add_var(
            "ltoe".to_string(),
            Data::new(ltgtoe_function("ltoe".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l <= r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) <= r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l <= r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l <= r,
            })),
        )
        .add_var(
            "gtoe".to_string(),
            Data::new(ltgtoe_function("gtoe".to_string(), |l, r| match (l, r) {
                (IntOrFloatOrNothing::Nothing, _) | (_, IntOrFloatOrNothing::Nothing) => true,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Int(r)) => l >= r,
                (IntOrFloatOrNothing::Int(l), IntOrFloatOrNothing::Float(r)) => (l as f64) >= r,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Int(r)) => l >= r as f64,
                (IntOrFloatOrNothing::Float(l), IntOrFloatOrNothing::Float(r)) => l >= r,
            })),
        )
        .add_var(
            "parse_float".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&Type::new(data::string::StringT)) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::float::FloatT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        Err(format!("parse_float called on non-string type").into())
                    }
                }),
                run: Arc::new(|a, _i| {
                    Ok(
                        if let Ok(n) = a
                            .get()
                            .as_any()
                            .downcast_ref::<data::string::String>()
                            .unwrap()
                            .0
                            .parse()
                        {
                            Data::one_tuple(Data::new(data::float::Float(n)))
                        } else {
                            Data::empty_tuple()
                        },
                    )
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "parse_int".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&Type::new(data::string::StringT)) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::int::IntT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        Err(format!("parse_float called on non-string type").into())
                    }
                }),
                run: Arc::new(|a, _i| {
                    Ok(
                        if let Ok(n) = a
                            .get()
                            .as_any()
                            .downcast_ref::<data::string::String>()
                            .unwrap()
                            .0
                            .parse()
                        {
                            Data::one_tuple(Data::new(data::int::Int(n)))
                        } else {
                            Data::empty_tuple()
                        },
                    )
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "signum".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&Type::newm(vec![
                        Arc::new(data::int::IntT),
                        Arc::new(data::float::FloatT),
                    ])) {
                        Ok(Type::new(data::int::IntT))
                    } else {
                        Err(format!("signum called on non-number type").into())
                    }
                }),
                run: Arc::new(|a, _i| {
                    Ok(Data::new(data::int::Int(
                        if let Some(n) = a.get().as_any().downcast_ref::<data::int::Int>() {
                            n.0.signum()
                        } else if let Some(n) =
                            a.get().as_any().downcast_ref::<data::float::Float>()
                        {
                            if n.0 > 0.0 {
                                1
                            } else if n.0 < 0.0 {
                                -1
                            } else {
                                0
                            }
                        } else {
                            return Err("called signum on non-number type".into());
                        },
                    )))
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "div".to_string(),
            Data::new(two_num_tuple_to_num(
                "div",
                |l, r| {
                    l.checked_div(r)
                        .ok_or_else(|| CheckError::from("attempted to divide by zero"))
                },
                |l, r| Ok(l as f64 / r),
                |l, r| Ok(l / r as f64),
                |l, r| Ok(l / r),
            )),
        )
        .add_var(
            "pow".to_string(),
            Data::new(two_num_tuple_to_num(
                "pow",
                |l, r| Ok(l.pow(r.try_into().unwrap_or(u32::MAX))),
                |l, r| Ok((l as f64).powf(r)),
                |l, r| {
                    Ok(if let Ok(r) = r.try_into() {
                        l.powi(r)
                    } else {
                        l.powf(r as f64)
                    })
                },
                |l, r| Ok(l.powf(r)),
            )),
        )
        .add_var(
            "modulo".to_string(),
            Data::new(two_num_tuple_to_num(
                "modulo",
                |l, r| {
                    l.checked_rem(r).ok_or_else(|| {
                        CheckError::from(
                            "called modulo on two integers, and the second one was zero",
                        )
                    })
                },
                |l, r| Ok(l as f64 % r),
                |l, r| Ok(l % r as f64),
                |l, r| Ok(l % r),
            )),
        )
        .add_var(
            "sum".to_string(),
            Data::new(num_iter_to_num("sum", Ok(0), |a, v| match (a, v) {
                (Ok(a), Ok(v)) => Ok(a + v),
                (Ok(a), Err(v)) => Err(a as f64 + v),
                (Err(a), Ok(v)) => Err(a + v as f64),
                (Err(a), Err(v)) => Err(a + v),
            })),
        )
        .add_var(
            "subtract".to_string(),
            Data::new(two_num_tuple_to_num(
                "subtract",
                |l, r| Ok(l - r),
                |l, r| Ok(l as f64 - r),
                |l, r| Ok(l - r as f64),
                |l, r| Ok(l - r),
            )),
        )
        .add_var(
            "product".to_string(),
            Data::new(num_iter_to_num("sum", Ok(1), |a, v| match (a, v) {
                (Ok(a), Ok(v)) => Ok(a * v),
                (Ok(a), Err(v)) => Err(a as f64 * v),
                (Err(a), Ok(v)) => Err(a * v as f64),
                (Err(a), Err(v)) => Err(a * v),
            })),
        )
    }
}

fn num_iter_to_num(
    func_name: &'static str,
    init: Result<isize, f64>,
    func: impl Fn(Result<isize, f64>, Result<isize, f64>) -> Result<isize, f64> + Send + Sync + 'static,
) -> data::function::Function {
    data::function::Function {
        info: program::run::Info::neverused(),
        info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
        out: Arc::new(move |a, _i| {
            if let Some(a) = a.iterable() {
                let int_type = Type::new(data::int::IntT);
                if a.is_included_in(&int_type) {
                    Ok(int_type)
                } else {
                    let float_type = Type::new(data::float::FloatT);
                    if a.is_included_in(&float_type) {
                        Ok(float_type)
                    } else {
                        let int_float_type = Type::newm(vec![
                            Arc::new(data::int::IntT),
                            Arc::new(data::float::FloatT),
                        ]);
                        if a.is_included_in(&int_float_type) {
                            Ok(int_float_type)
                        } else {
                            Err(format!("argument passed to {func_name} must be an iterator over values of type Int/String, but was an iterator over values of type {a}.").into())
                        }
                    }
                }
            } else {
                Err(format!("argument passed to {func_name} must be an iterator").into())
            }
        }),
        run: Arc::new(move |a, _i| {
            let mut out = init;
            for v in a.get().iterable().unwrap() {
                let v = v?;
                let v = v.get();
                let v = v.as_any();
                let v = v
                    .downcast_ref::<data::int::Int>()
                    .map(|v| Ok(v.0))
                    .unwrap_or_else(|| {
                        Err(v
                            .downcast_ref::<data::float::Float>()
                            .expect("value used in num-iterator function was not a number")
                            .0)
                    });
                out = func(out, v);
            }
            Ok(match out {
                Ok(v) => Data::new(data::int::Int(v)),
                Err(v) => Data::new(data::float::Float(v)),
            })
        }),
        inner_statements: None,
    }
}

/// (int, int) -> int
/// (int, float) -> float
/// (float, int) -> float
/// (float, float) -> float
fn two_num_tuple_to_num(
    func_name: &'static str,
    func_ii: impl Fn(isize, isize) -> Result<isize, CheckError> + Send + Sync + 'static,
    func_if: impl Fn(isize, f64) -> Result<f64, CheckError> + Send + Sync + 'static,
    func_fi: impl Fn(f64, isize) -> Result<f64, CheckError> + Send + Sync + 'static,
    func_ff: impl Fn(f64, f64) -> Result<f64, CheckError> + Send + Sync + 'static,
) -> data::function::Function {
    data::function::Function {
        info: program::run::Info::neverused(),
        info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
        out: Arc::new(|a, _i| two_tuple_to_num_impl_check(a, func_name)),
        run: Arc::new(move |a, _i| {
            two_tuple_to_num_impl_run(a, func_name, &func_ii, &func_if, &func_fi, &func_ff)
        }),
        inner_statements: None,
    }
}
fn two_tuple_to_num_impl_check(a: &Type, func_name: &str) -> Result<Type, CheckError> {
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
fn two_tuple_to_num_impl_run(
    a: Data,
    func_name: &'static str,
    func_ii: &(impl Fn(isize, isize) -> Result<isize, CheckError> + Send + Sync),
    func_if: &(impl Fn(isize, f64) -> Result<f64, CheckError> + Send + Sync),
    func_fi: &(impl Fn(f64, isize) -> Result<f64, CheckError> + Send + Sync),
    func_ff: &(impl Fn(f64, f64) -> Result<f64, CheckError> + Send + Sync),
) -> Result<Data, CheckError> {
    if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
        let left = t.0[0].get();
        let right = t.0[1].get();
        let (left, right) = (left.as_any(), right.as_any());
        Ok(
            match (
                left.downcast_ref::<data::int::Int>(),
                left.downcast_ref::<data::float::Float>(),
                right.downcast_ref::<data::int::Int>(),
                right.downcast_ref::<data::float::Float>(),
            ) {
                (Some(data::int::Int(l)), None, Some(data::int::Int(r)), None) => {
                    Data::new(data::int::Int(func_ii(*l, *r)?))
                }
                (Some(data::int::Int(l)), None, None, Some(data::float::Float(r))) => {
                    Data::new(data::float::Float(func_if(*l, *r)?))
                }
                (None, Some(data::float::Float(l)), Some(data::int::Int(r)), None) => {
                    Data::new(data::float::Float(func_fi(*l, *r)?))
                }
                (None, Some(data::float::Float(l)), None, Some(data::float::Float(r))) => {
                    Data::new(data::float::Float(func_ff(*l, *r)?))
                }
                _ => {
                    return Err(format!(
                    "at least one of the arguments to {func_name} were neither an int nor a float"
                )
                    .into())
                }
            },
        )
    } else {
        return Err(format!("argument to {func_name} was not a tuple").into());
    }
}

fn ltgtoe_function(
    func_name: String,
    op: impl Fn(IntOrFloatOrNothing, IntOrFloatOrNothing) -> bool + Send + Sync + 'static,
) -> data::function::Function {
    data::function::Function {
        info: program::run::Info::neverused(),
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
                let item = item?;
                let item = item.get();
                let new = if let Some(data::int::Int(v)) = item.as_any().downcast_ref() {
                    IntOrFloatOrNothing::Int(*v)
                } else if let Some(data::float::Float(v)) = item.as_any().downcast_ref() {
                    IntOrFloatOrNothing::Float(*v)
                } else {
                    return Err(
                        "one of the (l/g)t[oe] function argument iterator elements were neither int nor float".into(),
                    );
                };
                if op(prev, new) {
                    prev = new;
                } else {
                    return Ok(Data::new(data::bool::Bool(false)));
                }
            }
            Ok(Data::new(data::bool::Bool(true)))
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
