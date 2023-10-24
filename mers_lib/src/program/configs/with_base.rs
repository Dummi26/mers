use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, MersType, Type},
    program::run::{CheckError, CheckInfo, Info},
};

use super::Config;

impl Config {
    /// `deref: fn` clones the value from a reference
    /// `eq: fn` returns true if all the values are equal, otherwise false.
    /// `loop: fn` runs a function until it returns (T) instead of (), then returns T.
    /// `try: fn` runs the first valid function with the argument. usage: (arg, (f1, f2, f3)).try
    /// NOTE: try's return type may miss some types that can actually happen when using it on tuples, so... don't do ((a, b), (f1, any -> ())).try unless f1 also returns ()
    /// `len: fn` gets the length of strings or tuples
    /// `panic: fn` exits the program with the given exit code
    pub fn with_base(self) -> Self {
        self.add_var("try".to_string(), Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                let mut out = Type::empty();
                for t in a.types.iter() {
                    if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                        if t.0.len() != 2 {
                            return Err(format!("cannot use try with tuple argument where len != 2 (got len {})", t.0.len()).into());
                        }
                        let arg_type = &t.0[0];
                        let functions = &t.0[1];
                        for arg_type in arg_type.subtypes_type().types.iter() {
                            let arg_type = Type::newm(vec![arg_type.clone()]);
                            // possibilities for the tuple (f1, f2, f3, ..., fn)
                            for ft in functions.types.iter() {
                                let mut tuple_fallible = true;
                                let mut tuple_possible = false;
                                if let Some(ft) = ft.as_any().downcast_ref::<data::tuple::TupleT>() {
                                    // f1, f2, f3, ..., fn
                                    let mut func_errors = vec![];
                                    for ft in ft.0.iter() {
                                        let mut func_fallible = false;
                                        // possibilities for f_
                                        for ft in ft.types.iter() {
                                            if let Some(ft) = ft.as_any().downcast_ref::<data::function::FunctionT>() {
                                                func_errors.push(match ft.0(&arg_type) {
                                                    Err(e) => {
                                                        func_fallible = true;
                                                        Some(e)
                                                    }
                                                    Ok(o) => {
                                                        tuple_possible = true;
                                                        for t in o.types {
                                                            out.add(t);
                                                        }
                                                        None
                                                    },
                                                });
                                            } else {
                                                return Err(format!("try: arguments f1-fn must be functions").into());
                                            }
                                        }
                                        // found a function that won't fail for this arg_type!
                                        if !func_fallible {
                                            tuple_fallible = false;
                                            if tuple_possible {
                                                break;
                                            }
                                        }
                                    }
                                    if tuple_fallible || !tuple_possible {
                                        // if the argument is {arg_type}, there is no infallible function. add a fallback function to handle this case!
                                        let mut e = CheckError::new()
                                            .msg(format!("if the argument is {arg_type}, there is no infallible function."))
                                            .msg(format!("Add a fallback function to handle this case!"));
                                        for (i, err) in func_errors.into_iter().enumerate() {
                                            if let Some(err) = err {
                                                e = e
                                                    .msg(format!("Error for function #{}:", i + 1))
                                                    .err(err);
                                            }
                                        }
                                        return Err(e);
                                    }
                                } else {
                                    return Err(format!("try: argument must be (arg, (f1, f2, f3, ..., fn))").into());
                                }
                            }
                        }
                    } else {
                        return Err(format!("cannot use try with non-tuple argument").into());
                    }
                }
                Ok(out)
            }),
            run: Arc::new(|a, _i|  {
                let tuple = a.get();
                let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().expect("try: not a tuple");
                let arg = &tuple.0[0];
                let funcs = tuple.0[1].get();
                let funcs = funcs.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                for func in funcs.0.iter() {
                    let func = func.get();
                    let func = func.as_any().downcast_ref::<data::function::Function>().unwrap();
                    if func.check(&arg.get().as_type()).is_ok() {
                        return func.run(arg.clone());
                    }
                }
                unreachable!("try: no function found")
            })
        }))
            .add_var("panic".to_string(), Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.is_included_in(&data::int::IntT) {
                Ok(Type::empty())
            } else {
                Err(format!("cannot call exit with non-int argument").into())
            }),
            run: Arc::new(|a, _i|  {
                std::process::exit(a.get().as_any().downcast_ref::<data::int::Int>().map(|i| i.0 as _).unwrap_or(1));
            })
        }))
            .add_var(
            "len".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for t in &a.types {
                        if t.as_any().downcast_ref::<data::string::StringT>().is_none() && t.as_any().downcast_ref::<data::tuple::TupleT>().is_none() {
                            return Err(format!("cannot get length of {t} (must be a tuple or a string)").into());
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
                                        return Err(format!("called loop with funcion that might return a tuple of length > 1").into());
                                    } else if let Some(v) = t.0.first() {
                                        o.add(Arc::new(v.clone()))
                                    }
                                } else {
                                    return Err(format!("called loop with funcion that might return something other than a tuple").into());
                                }
                            }
                        } else {
                            return Err(format!("called loop on a non-function").into());
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
                                return Err(format!("called eq on non-iterable").into())
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
                out: Arc::new(|a, _i| if let Some(v) = a.dereference() { Ok(v) } else { Err(format!("cannot dereference type {a}").into())
                }),
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
