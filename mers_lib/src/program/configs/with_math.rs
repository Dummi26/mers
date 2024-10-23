use std::{ops::Rem, sync::Arc};

use crate::data::{
    self,
    function::Function,
    int::{INT_MAX, INT_MIN},
    Data, MersTypeWInfo, Type,
};

use super::{
    gen::{
        function::{fun, func, Funcs, StaticMersFunc},
        IntR, OneOf, OneOrNone,
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
            func(|n: &str, _| {
                Ok(OneOrNone(
                    n.parse::<isize>().ok().map(IntR::<INT_MIN, INT_MAX>),
                ))
            }),
        )
        .add_var(
            "lt",
            func(
                |v: (
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                ),
                 _| {
                    Ok(match v {
                        (OneOf::A(a), OneOf::A(b)) => a.0 < b.0,
                        (OneOf::A(a), OneOf::B(b)) => (a.0 as f64) < b,
                        (OneOf::B(a), OneOf::A(b)) => a < (b.0 as f64),
                        (OneOf::B(a), OneOf::B(b)) => a < b,
                    })
                },
            ),
        )
        .add_var(
            "gt",
            func(
                |v: (
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                ),
                 _| {
                    Ok(match v {
                        (OneOf::A(a), OneOf::A(b)) => a.0 > b.0,
                        (OneOf::A(a), OneOf::B(b)) => (a.0 as f64) > b,
                        (OneOf::B(a), OneOf::A(b)) => a > (b.0 as f64),
                        (OneOf::B(a), OneOf::B(b)) => a > b,
                    })
                },
            ),
        )
        .add_var(
            "ltoe",
            func(
                |v: (
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                ),
                 _| {
                    Ok(match v {
                        (OneOf::A(a), OneOf::A(b)) => a.0 <= b.0,
                        (OneOf::A(a), OneOf::B(b)) => (a.0 as f64) <= b,
                        (OneOf::B(a), OneOf::A(b)) => a <= (b.0 as f64),
                        (OneOf::B(a), OneOf::B(b)) => a <= b,
                    })
                },
            ),
        )
        .add_var(
            "gtoe",
            func(
                |v: (
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                    OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                ),
                 _| {
                    Ok(match v {
                        (OneOf::A(a), OneOf::A(b)) => a.0 >= b.0,
                        (OneOf::A(a), OneOf::B(b)) => (a.0 as f64) >= b,
                        (OneOf::B(a), OneOf::A(b)) => a >= (b.0 as f64),
                        (OneOf::B(a), OneOf::B(b)) => a >= b,
                    })
                },
            ),
        )
        .add_var(
            "signum",
            func(|n: OneOf<IntR<INT_MIN, INT_MAX>, f64>, _| {
                Ok(IntR::<-1, 1>(match n {
                    OneOf::A(n) => n.0.signum(),
                    OneOf::B(n) => {
                        if n > 0.0 {
                            1
                        } else if n < 0.0 {
                            -1
                        } else {
                            0
                        }
                    }
                }))
            }),
        )
        .add_var(
            "add",
            func_math_op(
                "add",
                |a, b| Some(a + b),
                false,
                |a, b| a.checked_add(b),
                |a, b| {
                    if a.0.checked_add(b.0).is_some()
                        || a.0.checked_add(b.1).is_some()
                        || a.1.checked_add(b.0).is_some()
                        || a.1.checked_add(b.1).is_some()
                    {
                        Some((
                            vec![Arc::new(data::int::IntT(
                                a.0.saturating_add(b.0),
                                a.1.saturating_add(b.1),
                            ))],
                            a.0.checked_add(b.0).is_none() || a.1.checked_add(b.1).is_none(),
                        ))
                    } else {
                        None
                    }
                },
                None,
            ),
        )
        .add_var(
            "sub",
            func_math_op(
                "sub",
                |a, b| Some(a - b),
                false,
                |a, b| a.checked_sub(b),
                |a, b| {
                    if a.0.checked_sub(b.0).is_some()
                        || a.0.checked_sub(b.1).is_some()
                        || a.1.checked_sub(b.0).is_some()
                        || a.1.checked_sub(b.1).is_some()
                    {
                        Some((
                            vec![Arc::new(data::int::IntT(
                                a.0.saturating_sub(b.1),
                                a.1.saturating_sub(b.0),
                            ))],
                            a.0.checked_sub(b.1).is_none() || a.1.checked_sub(b.0).is_none(),
                        ))
                    } else {
                        None
                    }
                },
                None,
            ),
        )
        .add_var(
            "mul",
            func_math_op(
                "mul",
                |a, b| Some(a * b),
                false,
                |a, b| a.checked_mul(b),
                |a, b| {
                    let (mut min, mut max, mut fails) = (None, None, false);
                    let mut minmax = |v| {
                        if min.is_none() || v < min.unwrap() {
                            min = Some(v);
                        }
                        if max.is_none() || v > max.unwrap() {
                            max = Some(v);
                        }
                    };
                    let mut trymul = |a: isize, b: isize| {
                        a.checked_mul(b).unwrap_or_else(|| {
                            fails = true;
                            match a.signum() * b.signum() {
                                0 => 0,
                                ..=-1 => isize::MIN,
                                1.. => isize::MAX,
                            }
                        })
                    };
                    minmax(trymul(a.0, b.0));
                    minmax(trymul(a.0, b.1));
                    minmax(trymul(a.1, b.0));
                    minmax(trymul(a.1, b.1));
                    if let (Some(min), Some(max)) = (min, max) {
                        Some((vec![Arc::new(data::int::IntT(min, max))], fails))
                    } else {
                        None
                    }
                },
                None,
            ),
        )
        .add_var(
            "div",
            func_math_op(
                "div",
                |a, b| Some(a / b),
                false,
                |a, b| a.checked_div(b),
                |a, b| {
                    let mut minmax = (None, None, None, None, b.0 <= 0 && 0 <= b.1);
                    if a.0 < 0 {
                        for_a(a.0, b, &mut minmax);
                        for_a(a.1.min(-1), b, &mut minmax);
                    }
                    if a.1 > 0 {
                        for_a(a.1, b, &mut minmax);
                        for_a(a.0.max(1), b, &mut minmax);
                    }
                    if a.0 <= 0 && 0 <= a.1 {
                        for_a(0, b, &mut minmax);
                    }
                    fn for_a(
                        a: isize,
                        b: &data::int::IntT,
                        minmax: &mut (
                            Option<isize>,
                            Option<isize>,
                            Option<isize>,
                            Option<isize>,
                            bool,
                        ),
                    ) {
                        if b.0 < 0 {
                            for_ab(a, b.0, minmax);
                            for_ab(a, b.1.min(-1), minmax);
                        }
                        if b.1 >= 0 {
                            for_ab(a, b.1, minmax);
                            for_ab(a, b.0.max(1), minmax);
                        }
                    }
                    fn for_ab(
                        a: isize,
                        b: isize,
                        minmax: &mut (
                            Option<isize>,
                            Option<isize>,
                            Option<isize>,
                            Option<isize>,
                            bool,
                        ),
                    ) {
                        if a.checked_div(b).is_none() {
                            minmax.4 = true;
                        }
                        if let Some(v) = a.checked_div(b).or_else(|| {
                            if b != 0 {
                                Some(a.saturating_div(b))
                            } else {
                                None
                            }
                        }) {
                            if v <= 0 {
                                for_mm(v, &mut minmax.0, &mut minmax.1);
                            }
                            if v >= 0 {
                                for_mm(v, &mut minmax.2, &mut minmax.3);
                            }
                        } else {
                        }
                    }
                    fn for_mm(v: isize, min: &mut Option<isize>, max: &mut Option<isize>) {
                        if min.is_none() || v < min.unwrap() {
                            *min = Some(v);
                        }
                        if max.is_none() || v > max.unwrap() {
                            *max = Some(v);
                        }
                    }
                    match minmax {
                        (Some(w), Some(x), Some(y), Some(z), f) if x == y => {
                            Some((vec![Arc::new(data::int::IntT(w, z))], f))
                        }
                        (Some(w), Some(x), Some(y), Some(z), f) => Some((
                            vec![
                                Arc::new(data::int::IntT(w, x)),
                                Arc::new(data::int::IntT(y, z)),
                            ],
                            f,
                        )),
                        (Some(w), Some(x), _, _, f) => {
                            Some((vec![Arc::new(data::int::IntT(w, x))], f))
                        }
                        (_, _, Some(y), Some(z), f) => {
                            Some((vec![Arc::new(data::int::IntT(y, z))], f))
                        }
                        (_, _, _, _, _) => None,
                    }
                },
                None,
            ),
        )
        .add_var(
            "modulo",
            func_math_op(
                "modulo",
                |a, b| Some(a.rem_euclid(b)),
                false,
                |a, b| a.checked_rem_euclid(b),
                |a, b| {
                    if b.0 == 0 && b.1 == 0 {
                        None
                    } else {
                        let can_fail =
                            b.0 <= 0 && 0 <= b.1 || (a.0 == isize::MIN && b.0 <= -1 && -1 <= b.1);
                        Some((
                            vec![Arc::new(data::int::IntT(0, {
                                let mut max = b.1.abs();
                                if max > 0 {
                                    max -= 1;
                                }
                                if a.0 < 0 {
                                    max
                                } else {
                                    max.min(a.1)
                                }
                            }))],
                            can_fail,
                        ))
                    }
                },
                None,
            ),
        )
        .add_var(
            "remainder",
            func_math_op(
                "remainder",
                |a, b| Some(a.rem(b)),
                false,
                |a, b| a.checked_rem(b),
                |a, b| {
                    if b.0 == 0 && b.1 == 0 {
                        None
                    } else {
                        let can_fail =
                            b.0 <= 0 && 0 <= b.1 || (a.0 == isize::MIN && b.0 <= -1 && -1 <= b.1);
                        Some((
                            vec![Arc::new({
                                let mut max = b.1.abs();
                                if max > 0 {
                                    max -= 1;
                                }
                                data::int::IntT(-max, max)
                            })],
                            can_fail,
                        ))
                    }
                },
                None,
            ),
        )
        .add_var(
            "abs",
            Funcs(
                fun(|v: IntR<INT_MIN, INT_MAX>, _| {
                    Ok(IntR::<INT_MIN, INT_MAX>(v.0.saturating_abs()))
                }),
                fun(|v: f64, _| Ok(v.abs())),
            )
            .mers_func(),
        )
        .add_var(
            "pow",
            Funcs(
                fun(|(l, r): (IntR<INT_MIN, INT_MAX>, IntR<0, INT_MAX>), _| {
                    Ok(IntR::<INT_MIN, INT_MAX>(
                        l.0.saturating_pow(r.0.try_into().unwrap_or(u32::MAX)),
                    ))
                }),
                fun(
                    |(l, r): (
                        OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                        OneOf<IntR<INT_MIN, INT_MAX>, f64>,
                    ),
                     _| {
                        let l = match l {
                            OneOf::A(v) => v.0 as f64,
                            OneOf::B(v) => v,
                        };
                        Ok(match r {
                            OneOf::A(r) => {
                                if let Ok(r) = r.0.try_into() {
                                    l.powi(r)
                                } else {
                                    l.powf(r.0 as f64)
                                }
                            }
                            OneOf::B(r) => l.powf(r),
                        })
                    },
                ),
            )
            .mers_func(),
        )
        .add_var(
            "min",
            Function::new_generic(
                |a, i| {
                    let mut o = Type::empty();
                    for a in &a.types {
                        if let Some(t) = a.as_any().downcast_ref::<data::tuple::TupleT>().filter(|v| v.0.len() == 2) {
                            let (a, b) = (&t.0[0], &t.0[1]);
                            let mut float = false;
                            for a in &a.types {
                                if let Some(a) = a.as_any().downcast_ref::<data::int::IntT>().map(Ok).or_else(|| a.as_any().downcast_ref::<data::float::FloatT>().map(Err)) {
                                    for b in &b.types {
                                        if let Some(b) = b.as_any().downcast_ref::<data::int::IntT>().map(Ok).or_else(|| b.as_any().downcast_ref::<data::float::FloatT>().map(Err)) {
                                            match (a, b) {
                                                (Ok(a), Ok(b)) => {
                                                    o.add(Arc::new(data::int::IntT(a.0.min(b.0), a.1.min(b.1))));
                                                },
                                                _ => float = true,
                                            }
                                        } else {
                                            return Err(format!("called `min` on a 2-tuple, `{}`, containing a non-`Int/Float` second element.", t.with_info(i)).into());
                                        }
                                    }
                                } else {
                                    return Err(format!("called `min` on a 2-tuple, `{}`, containing a non-`Int/Float` first element.", t.with_info(i)).into());
                                }
                            }
                            if float {
                                o.add(Arc::new(data::float::FloatT));
                            }
                        } else if let Some(a) = a.iterable() {
                            let mut int = false;
                            let mut float = false;
                            for e in &a.types {
                                if  e.as_any().is::<data::int::IntT>() {
                                    int = true;
                                } else if e.as_any().is::<data::float::FloatT>() {
                                    float = true;
                                } else {
                                    return Err(format!("called `min` on an iterator over elements of type {}, which is not `Int/Float`", a.with_info(i)).into());
                                }
                            }
                            if int {
                                o.add(Arc::new(data::int::IntT(INT_MIN, INT_MAX)));
                            }
                            if float {
                                o.add(Arc::new(data::float::FloatT));
                            }
                            o.add(Arc::new(data::tuple::TupleT(vec![])));
                        } else {
                            return Err(format!(
                                "cannot call `min` on non-iterable type {}",
                                a.with_info(i)
                            )
                            .into());
                        }
                    }
                    Ok(o)
                },
                |a, i| {
                    let mut min_int = None;
                    let mut min_float = None;
                    for a in a.get().iterable(&i.global).expect("called `min` on non-itereable") {
                        let a = a?;
                        let a = a.get();
                        let a = a.as_any().downcast_ref::<data::int::Int>().map(|v| Ok(v.0)).or_else(|| a.as_any().downcast_ref::<data::float::Float>().map(|v| Err(v.0))).expect("found non-Int/Float element in argument to `min`");
                        match a {
                            Ok(a) => if min_int.is_none() || a < min_int.unwrap() { min_int = Some(a); },
                            Err(a) => if min_float.is_none() || a < min_float.unwrap() { min_float = Some(a); },
                        }
                    }
                    Ok(match (min_float, min_int) {
                        (Some(a), Some(b)) => if a < b as f64  { Data::new(data::float::Float(a)) } else { Data::new(data::int::Int(b))},
                        (Some(a), None) => Data::new(data::float::Float(a)),
                        (None, Some(b)) => Data::new(data::int::Int(b)),
                        (None, None) => Data::empty_tuple(),
                    })
                },
            ),
        )
        .add_var(
            "max",
            Function::new_generic(
                |a, i| {
                    let mut o = Type::empty();
                    for a in &a.types {
                        if let Some(t) = a.as_any().downcast_ref::<data::tuple::TupleT>().filter(|v| v.0.len() == 2) {
                            let (a, b) = (&t.0[0], &t.0[1]);
                            let mut float = false;
                            for a in &a.types {
                                if let Some(a) = a.as_any().downcast_ref::<data::int::IntT>().map(Ok).or_else(|| a.as_any().downcast_ref::<data::float::FloatT>().map(Err)) {
                                    for b in &b.types {
                                        if let Some(b) = b.as_any().downcast_ref::<data::int::IntT>().map(Ok).or_else(|| b.as_any().downcast_ref::<data::float::FloatT>().map(Err)) {
                                            match (a, b) {
                                                (Ok(a), Ok(b)) => {
                                                    o.add(Arc::new(data::int::IntT(a.0.max(b.0), a.1.max(b.1))));
                                                },
                                                _ => float = true,
                                            }
                                        } else {
                                            return Err(format!("called `max` on a 2-tuple, `{}`, containing a non-`Int/Float` second element.", t.with_info(i)).into());
                                        }
                                    }
                                } else {
                                    return Err(format!("called `max` on a 2-tuple, `{}`, containing a non-`Int/Float` first element.", t.with_info(i)).into());
                                }
                            }
                            if float {
                                o.add(Arc::new(data::float::FloatT));
                            }
                        } else if let Some(a) = a.iterable() {
                            let mut int = false;
                            let mut float = false;
                            for e in &a.types {
                                if  e.as_any().is::<data::int::IntT>() {
                                    int = true;
                                } else if e.as_any().is::<data::float::FloatT>() {
                                    float = true;
                                } else {
                                    return Err(format!("called `max` on an iterator over elements of type {}, which is not `Int/Float`", a.with_info(i)).into());
                                }
                            }
                            if int {
                                o.add(Arc::new(data::int::IntT(INT_MIN, INT_MAX)));
                            }
                            if float {
                                o.add(Arc::new(data::float::FloatT));
                            }
                            o.add(Arc::new(data::tuple::TupleT(vec![])));
                        } else {
                            return Err(format!(
                                "cannot call `max` on non-iterable type {}",
                                a.with_info(i)
                            )
                            .into());
                        }
                    }
                    Ok(o)
                },
                |a, i| {
                    let mut max_int = None;
                    let mut max_float = None;
                    for a in a.get().iterable(&i.global).expect("called `min` on non-itereable") {
                        let a = a?;
                        let a = a.get();
                        let a = a.as_any().downcast_ref::<data::int::Int>().map(|v| Ok(v.0)).or_else(|| a.as_any().downcast_ref::<data::float::Float>().map(|v| Err(v.0))).expect("found non-Int/Float element in argument to `min`");
                        match a {
                            Ok(a) => if max_int.is_none() || a > max_int.unwrap() { max_int = Some(a); },
                            Err(a) => if max_float.is_none() || a > max_float.unwrap() { max_float = Some(a); },
                        }
                    }
                    Ok(match (max_float, max_int) {
                        (Some(a), Some(b)) => if a > b as f64  { Data::new(data::float::Float(a)) } else { Data::new(data::int::Int(b))},
                        (Some(a), None) => Data::new(data::float::Float(a)),
                        (None, Some(b)) => Data::new(data::int::Int(b)),
                        (None, None) => Data::empty_tuple(),
                    })
                },
            ),
        )
        .add_var(
            "as_float",
            func(|v: OneOf<IntR<INT_MIN, INT_MAX>, f64>, _| {
                Ok(match v {
                    OneOf::A(v) => v.0 as f64,
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
            func(|v: f64, _| -> Result<IntR<INT_MIN, INT_MAX>, _> { Ok(isize_from(v.round())) }),
        )
        .add_var(
            "ceil_to_int",
            func(|v: f64, _| -> Result<IntR<INT_MIN, INT_MAX>, _> { Ok(isize_from(v.ceil())) }),
        )
        .add_var(
            "floor_to_int",
            func(|v: f64, _| -> Result<IntR<INT_MIN, INT_MAX>, _> { Ok(isize_from(v.floor())) }),
        )
        .add_var(
            "truncate_to_int",
            func(|v: f64, _| -> Result<IntR<INT_MIN, INT_MAX>, _> { Ok(isize_from(v.trunc())) }),
        )
        .add_var(
            "round_ties_even_to_int".to_owned(),
            func(|v: f64, _| -> Result<IntR<INT_MIN, INT_MAX>, _> {
                Ok(isize_from(v.round_ties_even()))
            }),
        )
    }
}

const ISIZE_MAX_F: f64 = isize::MAX as _;
const ISIZE_MIN_F: f64 = isize::MIN as _;
fn isize_from(v: f64) -> IntR<INT_MIN, INT_MAX> {
    IntR(if v >= ISIZE_MAX_F {
        isize::MAX
    } else if v <= ISIZE_MIN_F {
        isize::MIN
    } else {
        v as isize
    })
}

/// for math operations which are fallible for integers (and maybe floats), like `+`, `-`, ...
///
/// iter_version, if some, is
/// - initial value. if `None`, `()` will be returned for empty iterators.
/// - op_int_may_fail. if `true` and the iterator may contain ints, adds `()` to the return type. this is the int equivalent to `op_float_may_fail`, which, for 2-tuples, is part of `op_int_ranges_with_may_fail_flag`.
fn func_math_op(
    funcname: &'static str,
    op_float: impl Fn(f64, f64) -> Option<f64> + Send + Sync + 'static,
    op_float_may_fail: bool,
    op_int: impl Fn(isize, isize) -> Option<isize> + Send + Sync + 'static,
    op_int_ranges_with_may_fail_flag: impl Fn(
            &data::int::IntT,
            &data::int::IntT,
        ) -> Option<(Vec<Arc</*data::int::IntT*/ dyn data::MersType>>, bool)>
        + Send
        + Sync
        + 'static,
    iter_version: Option<(Option<Result<isize, f64>>, bool)>,
) -> Function {
    Function::new_generic(
        move |a, i| {
            let mut o = Type::empty();
            let mut may_fail = false;
            for a in &a.types {
                'pre_iter: {
                    if let Some(a) = a.as_any().downcast_ref::<data::tuple::TupleT>() {
                        if a.0.len() == 2 {
                            let (a, b) = (&a.0[0], &a.0[1]);
                            for a in &a.types {
                                for b in &b.types {
                                    let a = a.as_any().downcast_ref::<data::int::IntT>().map(Ok).or_else(|| a.as_any().downcast_ref::<data::float::FloatT>().map(Err)).ok_or_else(|| format!("`{funcname}`: first argument must be `Int/Float`, but was `{}`.", a.with_info(i)))?;
                                    let b = b.as_any().downcast_ref::<data::int::IntT>().map(Ok).or_else(|| b.as_any().downcast_ref::<data::float::FloatT>().map(Err)).ok_or_else(|| format!("`{funcname}`: second argument must be `Int/Float`, but was `{}`.", b.with_info(i)))?;
                                    match (a, b) {
                                        (Ok(a), Ok(b)) => {
                                            if let Some((range, fail)) =
                                                op_int_ranges_with_may_fail_flag(a, b)
                                            {
                                                if fail {
                                                    may_fail = true;
                                                }
                                                o.add_all(&Type::newm(range));
                                            } else {
                                                may_fail = true;
                                                // always fails, no need to add any types
                                            }
                                        }
                                        _ => {
                                            if op_float_may_fail {
                                                may_fail = true;
                                            }
                                            o.add(Arc::new(data::float::FloatT));
                                        }
                                    }
                                }
                            }
                            break 'pre_iter;
                        }
                    }
                    // ITER VERSION
                    if let Some((init, f_int)) = &iter_version {
                        if let Some(it) = a.iterable() {
                            let mut is_int = false;
                            let mut is_float = false;
                            for a in &it.types {
                                if a.as_any().is::<data::int::IntT>() {
                                    is_int = true;
                                } else if a.as_any().is::<data::float::FloatT>() {
                                    is_float = true;
                                } else {
                                    return Err(format!(
                                    "cannot call `{funcname}` on an iterator over type `{}`, because it contains the non-`Int/Float` type {}.", it.with_info(i), a.with_info(i)
                                )
                                .into());
                                }
                            }
                            let (o_int, o_float) = match init {
                                None => {
                                    // if the iterator is empty
                                    may_fail = true;
                                    (is_int, is_float)
                                }
                                Some(Ok(_)) => {
                                    if is_int && *f_int {
                                        may_fail = true;
                                    }
                                    if is_float && op_float_may_fail {
                                        may_fail = true;
                                    }
                                    (true, is_float)
                                }
                                Some(Err(_)) => {
                                    if is_float && op_float_may_fail {
                                        may_fail = true;
                                    }
                                    (false, true)
                                }
                            };
                            if o_int {
                                o.add(Arc::new(data::tuple::TupleT(vec![Type::new(
                                    data::int::IntT(INT_MIN, INT_MAX),
                                )])));
                            }
                            if o_float {
                                o.add(Arc::new(data::tuple::TupleT(vec![Type::new(
                                    data::float::FloatT,
                                )])));
                            }
                        } else {
                            return Err(format!(
                                "cannot call `{funcname}` on non-iterable type `{}`",
                                a.with_info(i)
                            )
                            .into());
                        }
                    } else {
                        return Err(format!(
                            "cannot call `{funcname}` on non-2-tuple type `{}`",
                            a.with_info(i)
                        )
                        .into());
                    }
                }
            }
            if may_fail {
                o.add(Arc::new(data::tuple::TupleT(vec![])));
            }
            Ok(o)
        },
        move |a, i| {
            let a = a.get();
            Ok(
                if let Some(a) = &a
                    .as_any()
                    .downcast_ref::<data::tuple::Tuple>()
                    .map(|v| &v.0)
                    .filter(|v| v.len() == 2)
                {
                    let (a, b) = (&a[0], &a[1]);
                    let (a, b) = (a.get(), b.get());
                    let a = a
                        .as_any()
                        .downcast_ref::<data::int::Int>()
                        .map(Ok)
                        .or_else(|| a.as_any().downcast_ref::<data::float::Float>().map(Err))
                        .unwrap();
                    let b = b
                        .as_any()
                        .downcast_ref::<data::int::Int>()
                        .map(Ok)
                        .or_else(|| b.as_any().downcast_ref::<data::float::Float>().map(Err))
                        .unwrap();
                    if let Some(v) = match (a, b) {
                        (Ok(a), Ok(b)) => op_int(a.0, b.0).map(data::int::Int).map(Data::new),
                        (Ok(a), Err(b)) => op_float(a.0 as f64, b.0)
                            .map(data::float::Float)
                            .map(Data::new),
                        (Err(a), Ok(b)) => op_float(a.0, b.0 as f64)
                            .map(data::float::Float)
                            .map(Data::new),
                        (Err(a), Err(b)) => {
                            op_float(a.0, b.0).map(data::float::Float).map(Data::new)
                        }
                    } {
                        v
                    } else {
                        Data::empty_tuple()
                    }
                } else {
                    let (mut acc, _) = iter_version
                        .expect("no iter version for this math op, but argument not a 2-tuple...");
                    for a in a
                        .iterable(&i.global)
                        .expect("math op with iter version called on non-iterable")
                    {
                        let a = a?;
                        let a = a.get();
                        let a = a
                            .as_any()
                            .downcast_ref::<data::int::Int>()
                            .map(Ok)
                            .or_else(|| a.as_any().downcast_ref::<data::float::Float>().map(Err))
                            .unwrap();
                        acc = if let Some(acc) = acc {
                            match (acc, a) {
                                (Ok(a), Ok(b)) => op_int(a, b.0).map(Ok),
                                (Ok(a), Err(b)) => op_float(a as f64, b.0).map(Err),
                                (Err(a), Ok(b)) => op_float(a, b.0 as f64).map(Err),
                                (Err(a), Err(b)) => op_float(a, b.0).map(Err),
                            }
                        } else {
                            Some(a.map(|v| v.0).map_err(|v| v.0))
                        };
                    }
                    match acc {
                        None => Data::empty_tuple(),
                        Some(v) => match v {
                            Ok(v) => Data::new(data::int::Int(v)),
                            Err(v) => Data::new(data::float::Float(v)),
                        },
                    }
                },
            )
        },
    )
}
