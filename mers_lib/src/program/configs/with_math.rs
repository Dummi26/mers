use std::{
    ops::Rem,
    sync::{Arc, Mutex},
};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    program::{self, run::CheckInfo},
};

use super::{
    gen::{
        function::{fun, func, Funcs, StaticMersFunc},
        OneOf, OneOrNone,
    },
    Config,
};

impl Config {
    /// `add: fn` returns the sum of all the numbers in the tuple
    /// `sub: fn` returns the first number minus all the others
    /// `mul: fn` returns the product of all the numbers in the tuple
    /// `div: fn` returns a / b. Performs integer division if a and b are both integers.
    /// `remainder: fn` returns a % b
    /// `modulo: fn` returns a % b, where a % b >= 0
    /// `abs: fn` returns the absolute value of a, abs(a) or |a|, which is a for a >= 0 and -a for a < 0. For a==isize::MIN, returns isize::MAX, which is one less than the theoretical absolute value of isize::MIN
    /// `pow: fn` returns a^b or a**b.
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
    /// `round_as_float: fn` round ties (x.5) away from zero, return the result as a float
    /// `ceil_as_float: fn` round all numbers towards +infty, return the result as a float
    /// `floor_as_float: fn` round all numbers towards -infty, return the result as a float
    /// `truncate_as_float: fn` round all numbers towards 0, return the result as a float
    /// `round_ties_even_as_float: fn` round ties (x.5) to the nearest even number, return the result as a float
    /// `round_to_int: fn` round ties (x.5) away from zero, return the result as an Int (saturates at the Int boundaries, hence the to_int instead of as_int)
    /// `ceil_to_int: fn` round all numbers towards +infty, return the result as an Int (saturates at the Int boundaries, hence the to_int instead of as_int)
    /// `floor_to_int: fn` round all numbers towards -infty, return the result as an Int (saturates at the Int boundaries, hence the to_int instead of as_int)
    /// `truncate_to_int: fn` round all numbers towards 0, return the result as an Int (saturates at the Int boundaries, hence the to_int instead of as_int)
    /// `round_ties_even_to_int: fn` round ties (x.5) to the nearest even number, return the result as an Int (saturates at the Int boundaries, hence the to_int instead of as_int)
    pub fn with_math(self) -> Self {
        self.add_var(
            "parse_float",
            func(|n: &str, _| Ok(OneOrNone(n.parse::<f64>().ok()))),
        )
        .add_var(
            "parse_int",
            func(|n: &str, _| Ok(OneOrNone(n.parse::<isize>().ok()))),
        )
        .add_var(
            "lt",
            func(|v: (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                Ok(match v {
                    (OneOf::A(a), OneOf::A(b)) => a < b,
                    (OneOf::A(a), OneOf::B(b)) => (a as f64) < b,
                    (OneOf::B(a), OneOf::A(b)) => a < (b as f64),
                    (OneOf::B(a), OneOf::B(b)) => a < b,
                })
            }),
        )
        .add_var(
            "gt",
            func(|v: (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                Ok(match v {
                    (OneOf::A(a), OneOf::A(b)) => a > b,
                    (OneOf::A(a), OneOf::B(b)) => (a as f64) > b,
                    (OneOf::B(a), OneOf::A(b)) => a > (b as f64),
                    (OneOf::B(a), OneOf::B(b)) => a > b,
                })
            }),
        )
        .add_var(
            "ltoe",
            func(|v: (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                Ok(match v {
                    (OneOf::A(a), OneOf::A(b)) => a <= b,
                    (OneOf::A(a), OneOf::B(b)) => (a as f64) <= b,
                    (OneOf::B(a), OneOf::A(b)) => a <= (b as f64),
                    (OneOf::B(a), OneOf::B(b)) => a <= b,
                })
            }),
        )
        .add_var(
            "gtoe",
            func(|v: (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                Ok(match v {
                    (OneOf::A(a), OneOf::A(b)) => a >= b,
                    (OneOf::A(a), OneOf::B(b)) => (a as f64) >= b,
                    (OneOf::B(a), OneOf::A(b)) => a >= (b as f64),
                    (OneOf::B(a), OneOf::B(b)) => a >= b,
                })
            }),
        )
        .add_var(
            "signum",
            func(|n: OneOf<isize, f64>, _| {
                Ok(match n {
                    OneOf::A(n) => n.signum(),
                    OneOf::B(n) => {
                        if n > 0.0 {
                            1
                        } else if n < 0.0 {
                            -1
                        } else {
                            0
                        }
                    }
                })
            }),
        )
        .add_var(
            "add",
            num_iter_to_num("sum", Ok(0), |a, v| match (a, v) {
                (Ok(a), Ok(v)) => Ok(a + v),
                (Ok(a), Err(v)) => Err(a as f64 + v),
                (Err(a), Ok(v)) => Err(a + v as f64),
                (Err(a), Err(v)) => Err(a + v),
            }),
        )
        .add_var(
            "sub",
            Funcs(
                fun(|(n, d): (isize, isize), _| Ok(n.wrapping_sub(d))),
                fun(|(n, d): (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                    let n = match n {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    let d = match d {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    Ok(n - d)
                }),
            )
            .mers_func(),
        )
        .add_var(
            "mul",
            num_iter_to_num("sum", Ok(1), |a, v| match (a, v) {
                (Ok(a), Ok(v)) => Ok(a * v),
                (Ok(a), Err(v)) => Err(a as f64 * v),
                (Err(a), Ok(v)) => Err(a * v as f64),
                (Err(a), Err(v)) => Err(a * v),
            }),
        )
        .add_var(
            "div",
            Funcs(
                fun(|(n, d): (isize, isize), _| {
                    n.checked_div(d)
                        .ok_or_else(|| CheckError::from("attempted to divide by zero"))
                }),
                fun(|(n, d): (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                    let n = match n {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    let d = match d {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    Ok(n / d)
                }),
            )
            .mers_func(),
        )
        .add_var(
            "remainder",
            Funcs(
                fun(|(n, d): (isize, isize), _| {
                    n.checked_rem(d).ok_or_else(|| {
                        CheckError::from(
                            "attempted to calculate remainder with zero, or overflow occured",
                        )
                    })
                }),
                fun(|(n, d): (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                    let n = match n {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    let d = match d {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    Ok(n.rem(d))
                }),
            )
            .mers_func(),
        )
        .add_var(
            "modulo",
            Funcs(
                fun(|(n, d): (isize, isize), _| {
                    n.checked_rem_euclid(d).ok_or_else(|| {
                        CheckError::from(
                            "attempted to perform modulo with zero, or overflow occured",
                        )
                    })
                }),
                fun(|(n, d): (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                    let n = match n {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    let d = match d {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    Ok(n.rem_euclid(d))
                }),
            )
            .mers_func(),
        )
        .add_var(
            "abs",
            Funcs(
                fun(|v: isize, _| Ok(v.saturating_abs())),
                fun(|v: f64, _| Ok(v.abs())),
            )
            .mers_func(),
        )
        .add_var(
            "pow",
            Funcs(
                fun(|(l, r): (isize, isize), _| Ok(l.pow(r.try_into().unwrap_or(u32::MAX)))),
                fun(|(l, r): (OneOf<isize, f64>, OneOf<isize, f64>), _| {
                    let l = match l {
                        OneOf::A(v) => v as f64,
                        OneOf::B(v) => v,
                    };
                    Ok(match r {
                        OneOf::A(r) => {
                            if let Ok(r) = r.try_into() {
                                l.powi(r)
                            } else {
                                l.powf(r as f64)
                            }
                        }
                        OneOf::B(r) => l.powf(r),
                    })
                }),
            )
            .mers_func(),
        )
        .add_var(
            "as_float",
            func(|v: OneOf<isize, f64>, _| {
                Ok(match v {
                    OneOf::A(v) => v as f64,
                    OneOf::B(v) => v,
                })
            }),
        )
        .add_var(
            "round_as_float",
            func(|v: f64, _| -> Result<f64, _> { Ok(v.round()) }),
        )
        .add_var(
            "ceil_as_float",
            func(|v: f64, _| -> Result<f64, _> { Ok(v.ceil()) }),
        )
        .add_var(
            "floor_as_float",
            func(|v: f64, _| -> Result<f64, _> { Ok(v.floor()) }),
        )
        .add_var(
            "truncate_as_float",
            func(|v: f64, _| -> Result<f64, _> { Ok(v.trunc()) }),
        )
        .add_var(
            "round_ties_even_as_float",
            func(|v: f64, _| -> Result<f64, _> { Ok(v.round_ties_even()) }),
        )
        .add_var(
            "round_to_int",
            func(|v: f64, _| -> Result<isize, _> { Ok(isize_from(v.round())) }),
        )
        .add_var(
            "ceil_to_int",
            func(|v: f64, _| -> Result<isize, _> { Ok(isize_from(v.ceil())) }),
        )
        .add_var(
            "floor_to_int",
            func(|v: f64, _| -> Result<isize, _> { Ok(isize_from(v.floor())) }),
        )
        .add_var(
            "truncate_to_int",
            func(|v: f64, _| -> Result<isize, _> { Ok(isize_from(v.trunc())) }),
        )
        .add_var(
            "round_ties_even_to_int".to_owned(),
            func(|v: f64, _| -> Result<isize, _> { Ok(isize_from(v.round_ties_even())) }),
        )
    }
}

const ISIZE_MAX_F: f64 = isize::MAX as _;
const ISIZE_MIN_F: f64 = isize::MIN as _;
fn isize_from(v: f64) -> isize {
    if v >= ISIZE_MAX_F {
        isize::MAX
    } else if v <= ISIZE_MIN_F {
        isize::MIN
    } else {
        v as isize
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
        out: Ok(Arc::new(move |a, _i| {
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
        })),
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
