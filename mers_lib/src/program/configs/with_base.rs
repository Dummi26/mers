use std::{
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    program::run::{CheckInfo, Info},
};

use super::{
    gen::{
        function::{func, func_end, func_err},
        OneOf,
    },
    Config,
};

impl Config {
    /// `deref: fn` clones the value from a reference
    /// `mkref: fn` returns a reference to a copy of the value
    /// `eq: fn` returns true if all the values are equal, otherwise false.
    /// `len: fn` gets the length of strings or tuples
    /// `sleep: fn` sleeps for n seconds (pauses the current thread)
    /// `panic: fn` exits the program with the given exit code
    /// `lock_update: fn` locks the value of a reference so you can exclusively modify it: &var.lock_update(v -> (v, 1).sum)
    pub fn with_base(self) -> Self {
        self
            .add_var("lock_update", data::function::Function {
                info: Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, _i| {
                    for t in a.types.iter() {
                        if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                            if t.0.len() == 2 {
                                let arg_ref = &t.0[0];
                                if let Some(arg) = arg_ref.dereference() {
                                    let func = &t.0[1];
                                    for func_t in func.types.iter() {
                                        if let Some(f) = func_t.executable() {
                                            match f.o(&arg) {
                                                Ok(out) => {
                                                    if !out.is_included_in(&arg) {
                                                        return Err(format!("Function returns a value of type {out}, which isn't included in the type of the reference, {arg}.").into());
                                                    }
                                                },
                                                Err(e) => return Err(CheckError::new().msg_str(format!("Invalid argument type {arg} for function")).err(e)),
                                            }
                                        } else {
                                            return Err(format!("Arguments must be (reference, function)").into());
                                        }
                                    }
                                } else {
                                    return Err(format!("Arguments must be (reference, function), but {arg_ref} isn't a reference").into());
                                }
                            } else {
                                return Err(format!("Can't call lock_update on tuple type {t} with length != 2, which is part of the argument type {a}.").into());
                            }
                        } else {
                            return Err(format!("Can't call lock_update on non-tuple type {t}, which is part of the argument type {a}.").into());
                        }
                    }
                    Ok(Type::empty_tuple())
                })),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let a = a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                    let arg_ref = a.0[0].get();
                    let arg_ref = arg_ref.as_any().downcast_ref::<data::reference::Reference>().unwrap();
                    let mut arg = arg_ref.0.write().unwrap();
                    let func = a.0[1].get();
                    *arg = func.execute(arg.clone()).unwrap()?;
                    Ok(Data::empty_tuple())
                }),
                inner_statements: None,
            })
            .add_var("sleep", func(|dur: OneOf<isize, f64>, i| {
                let mut sleep_dur = match dur {
                    OneOf::A(dur) => Duration::from_secs(dur.max(0).try_into().unwrap_or(u64::MAX)),
                    OneOf::B(dur) => Duration::from_secs_f64(dur.max(0.0)),
                };
                // limit how long sleep can take
                if let Some(cutoff) = i.global.limit_runtime {
                    sleep_dur = sleep_dur.min(cutoff.saturating_duration_since(Instant::now()));
                }
                std::thread::sleep(sleep_dur);
                Ok(())
            }))
            .add_var("exit", func_end(|code: isize, _| {
                std::process::exit(code.try_into().unwrap_or(255));
            }))
            .add_var("panic", func_err(|message: &str, _| {
                CheckError::from(message)
            }))
            .add_var(
            "len",
            data::function::Function {
                info: Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, _i| {
                    for t in &a.types {
                        if t.as_any().downcast_ref::<data::string::StringT>().is_none() && t.as_any().downcast_ref::<data::tuple::TupleT>().is_none() && t.iterable().is_none() {
                            return Err(format!("cannot get length of {t} (must be a tuple, string or iterable)").into());
                        }
                    }
                    Ok(Type::new(data::int::IntT))
                })),
                run: Arc::new(|a, _i| {
                    Ok(Data::new(data::int::Int(if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        t.0.len() as _
                    } else if let Some(s) = a.get().as_any().downcast_ref::<data::string::String>() {
                        s.0.len() as _
                    } else if let Some(i) = a.get().iterable() {
                        // -1 if more elements than isize can represent
                        i.take(isize::MAX as usize + 1).count().try_into().unwrap_or(-1)
                    } else {
                        return Err("called len on {a:?}, which isn't a tuple or a string".into());
                    })))
                }),
                inner_statements: None,
            },
        )
        .add_var(
            "eq",
            data::function::Function {
                info: Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, _i| {
                    for t in &a.types {
                            if t.iterable().is_none() {
                                return Err(format!("called eq on non-iterable").into())
                            }
                    }
                        Ok(Type::new(data::bool::BoolT))
                    })),
                run: Arc::new(|a, _i| {
                    Ok(Data::new(data::bool::Bool(if let Some(mut i) = a.get().iterable() {
                        if let Some(f) = i.next() {
                            let f = f?;
                            let mut o = true;
                            for el in i {
                                let el = el?;
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
                    })))
                }),
                inner_statements: None,
            },
        )
        .add_var(
            "mkref",
            data::function::Function {
                info: Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, _i| Ok(Type::new(data::reference::ReferenceT(a.clone()))))),
                run: Arc::new(|a, _i| {
                    Ok(Data::new(data::reference::Reference(Arc::new(RwLock::new(a.clone())))))
                }),
                inner_statements: None,
            },
        )
        .add_var(
            "deref",
            data::function::Function {
                info: Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, _i| if let Some(v) = a.dereference() { Ok(v) } else { Err(format!("cannot dereference type {a}").into())
                })),
                run: Arc::new(|a, _i| {
                    if let Some(r) = a
                        .get()
                        .as_any()
                        .downcast_ref::<data::reference::Reference>()
                    {
                        Ok(r.0.write().unwrap().clone())
                    } else {
                        Err("called deref on non-reference".into())
                    }
                }),
                inner_statements: None,
            },
        )
    }
}
