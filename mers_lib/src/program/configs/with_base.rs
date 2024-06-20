use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    program::run::{CheckInfo, Info},
};

use super::Config;

impl Config {
    /// `deref: fn` clones the value from a reference
    /// `mkref: fn` returns a reference to a copy of the value
    /// `eq: fn` returns true if all the values are equal, otherwise false.
    /// `loop: fn` runs a function until it returns (T) instead of (), then returns T. Also works with ((), f) instead of f for ().loop(() -> { ... }) syntax, which may be more readable
    /// `try: fn` runs the first valid function with the argument. usage: (arg, (f1, f2, f3)).try
    /// NOTE: try's return type may miss some types that can actually happen when using it on tuples, so... don't do ((a, b), (f1, any -> ())).try unless f1 also returns ()
    /// `len: fn` gets the length of strings or tuples
    /// `sleep: fn` sleeps for n seconds (pauses the current thread)
    /// `panic: fn` exits the program with the given exit code
    /// `lock_update: fn` locks the value of a reference so you can exclusively modify it: &var.lock_update(v -> (v, 1).sum)
    pub fn with_base(self) -> Self {
        self
            // .add_var("try".to_string(), get_try(false))
            // .add_var("try_allow_unused".to_string(), get_try(true))
            .add_var("lock_update".to_string(), Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for t in a.types.iter() {
                        if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                            if t.0.len() == 2 {
                                let arg_ref = &t.0[0];
                                if let Some(arg) = arg_ref.dereference() {
                                    let func = &t.0[1];
                                    for func_t in func.types.iter() {
                                        if let Some(f) = func_t.as_any().downcast_ref::<data::function::FunctionT>() {
                                            match f.o(&arg) {
                                                Ok(out) => {
                                                    if !out.is_included_in(&arg) {
                                                        return Err(format!("Function returns a value of type {out}, which isn't included in the type of the reference, {arg}.").into());
                                                    }
                                                },
                                                Err(e) => return Err(CheckError::new().msg(format!("Invalid argument type {arg} for function")).err(e)),
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
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let a = a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                    let arg_ref = a.0[0].get();
                    let arg_ref = arg_ref.as_any().downcast_ref::<data::reference::Reference>().unwrap();
                    let mut arg = arg_ref.0.write().unwrap();
                    let func = a.0[1].get();
                    let func = func.as_any().downcast_ref::<data::function::Function>().unwrap();
                    *arg = func.run(arg.clone())?;
                    Ok(Data::empty_tuple())
                }),
                inner_statements: None,
            }))
            .add_var("sleep".to_string(), Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| if a.is_included_in(&Type::newm(vec![
                    Arc::new(data::int::IntT),
                    Arc::new(data::float::FloatT),
                ])) {
                        Ok(Type::empty_tuple())
                } else {
                        Err(format!("cannot call sleep with non-int or non-float argument.").into())
                    }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    std::thread::sleep(if let Some(data::int::Int(n)) = a.as_any().downcast_ref() {
                        Duration::from_secs(*n as _)
                    } else if let Some(data::float::Float(n)) = a.as_any().downcast_ref() {
                        Duration::from_secs_f64(*n)
                    } else {
                        return Err("sleep called on non-int/non-float".into());
                    });
                    Ok(Data::empty_tuple())
                }),
                inner_statements: None,
            }))
            .add_var("exit".to_string(), Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.is_included_in_single(&data::int::IntT) {
                Ok(Type::empty())
            } else {
                Err(format!("cannot call exit with non-int argument").into())
            }),
            run: Arc::new(|a, _i|  {
                std::process::exit(a.get().as_any().downcast_ref::<data::int::Int>().map(|i| i.0 as _).unwrap_or(1));
            }),
            inner_statements: None,
        }))
            .add_var("panic".to_string(), Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.is_included_in_single(&data::string::StringT) {
                Ok(Type::empty())
            } else {
                Err(format!("cannot call panic with non-string argument").into())
            }),
            run: Arc::new(|a, _i|  {
                Err(
                    a
                        .get()
                        .as_any()
                        .downcast_ref::<data::string::String>()
                        .map(|i| i.0.to_owned())
                        .unwrap_or_else(String::new).into()
                )
            }),
            inner_statements: None,
        }))
            .add_var(
            "len".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for t in &a.types {
                        if t.as_any().downcast_ref::<data::string::StringT>().is_none() && t.as_any().downcast_ref::<data::tuple::TupleT>().is_none() && t.iterable().is_none() {
                            return Err(format!("cannot get length of {t} (must be a tuple, string or iterable)").into());
                        }
                    }
                    Ok(Type::new(data::int::IntT))
                }),
                run: Arc::new(|a, _i| {
                    Ok(Data::new(data::int::Int(if let Some(t) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        t.0.len() as _
                    } else if let Some(s) = a.get().as_any().downcast_ref::<data::string::String>() {
                        s.0.len() as _
                    } else if let Some(i) = a.get().iterable() {
                            // -1 if more elements than isize can represent
                            i.take(isize::MAX as usize + 1).count() as isize
                    } else {
                        return Err("called len on {a:?}, which isn't a tuple or a string".into());
                    })))
                }),
                inner_statements: None,
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
            }),
        )
        .add_var(
            "mkref".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| Ok(Type::new(data::reference::ReferenceT(a.clone())))),
                run: Arc::new(|a, _i| {
                    Ok(Data::new(data::reference::Reference(Arc::new(RwLock::new(a.clone())))))
                }),
                inner_statements: None,
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
                        Ok(r.0.write().unwrap().clone())
                    } else {
                        Err("called deref on non-reference".into())
                    }
                }),
                inner_statements: None,
            }),
        )
    }
}

// fn get_try(allow_unused_functions: bool) -> Data {
//     Data::new(data::function::Function {
//         info: Arc::new(Info::neverused()),
//         info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
//         out: Arc::new(move |a, _i| {
//             let mut out = Type::empty();
//             for t in a.types.iter() {
//                 if let Some(outer_tuple) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
//                     if outer_tuple.0.len() != 2 {
//                         return Err(format!(
//                             "cannot use try with tuple argument where len != 2 (got len {})",
//                             outer_tuple.0.len()
//                         )
//                         .into());
//                     }
//                     let arg_type = &outer_tuple.0[0];
//                     let functions = &outer_tuple.0[1];
//                     let mut used_functions_and_errors = vec![];
//                     for arg_type in arg_type.subtypes_type().types.iter() {
//                         let arg_type = Type::newm(vec![arg_type.clone()]);
//                         // possibilities for the tuple (f1, f2, f3, ..., fn)
//                         for (fti, ft) in functions.types.iter().enumerate() {
//                             if used_functions_and_errors.len() <= fti {
//                                 used_functions_and_errors.push(vec![]);
//                             }
//                             let mut tuple_fallible = true;
//                             let mut tuple_possible = false;
//                             if let Some(ft) = ft.as_any().downcast_ref::<data::tuple::TupleT>() {
//                                 // f1, f2, f3, ..., fn
//                                 let mut func_errors = vec![];
//                                 let mut skip_checks = false;
//                                 for (fi, ft) in ft.0.iter().enumerate() {
//                                     if used_functions_and_errors[fti].len() <= fi {
//                                         used_functions_and_errors[fti].push(vec![]);
//                                     }
//                                     let mut func_fallible = false;
//                                     // possibilities for f_
//                                     for (fvi, ft) in ft.types.iter().enumerate() {
//                                         if let Some(ft) =
//                                             ft.as_any().downcast_ref::<data::function::FunctionT>()
//                                         {
//                                             if used_functions_and_errors[fti][fi].len() <= fvi {
//                                                 used_functions_and_errors[fti][fi]
//                                                     .push(Some(vec![]));
//                                             }
//                                             if !skip_checks {
//                                                 func_errors.push((
//                                                     fvi,
//                                                     match ft.o(&arg_type) {
//                                                         Err(e) => {
//                                                             func_fallible = true;
//                                                             if let Some(errs) =
//                                                                 &mut used_functions_and_errors[fti]
//                                                                     [fi][fvi]
//                                                             {
//                                                                 errs.push(e.clone());
//                                                             }
//                                                             Some(e)
//                                                         }
//                                                         Ok(o) => {
//                                                             used_functions_and_errors[fti][fi]
//                                                                 [fvi] = None;
//                                                             tuple_possible = true;
//                                                             for t in o.types {
//                                                                 out.add(t);
//                                                             }
//                                                             None
//                                                         }
//                                                     },
//                                                 ));
//                                             }
//                                         } else {
//                                             return Err(format!(
//                                                 "try: arguments f1-fn must be functions"
//                                             )
//                                             .into());
//                                         }
//                                     }
//                                     // found a function that won't fail for this arg_type!
//                                     if !func_fallible {
//                                         tuple_fallible = false;
//                                         if tuple_possible {
//                                             skip_checks = true;
//                                         }
//                                     }
//                                 }
//                                 if tuple_fallible || !tuple_possible {
//                                     // if the argument is {arg_type}, there is no infallible function. add a fallback function to handle this case!
//                                     let mut e = CheckError::new()
//                                             .msg(format!("if the argument is {arg_type}, there is no infallible function."))
//                                             .msg(format!("Add a function to handle this case!"));
//                                     for (i, err) in func_errors.into_iter() {
//                                         if let Some(err) = err {
//                                             e = e
//                                                 .msg(format!("Error for function #{}:", i + 1))
//                                                 .err(err);
//                                         }
//                                     }
//                                     return Err(e);
//                                 }
//                             } else {
//                                 return Err(format!(
//                                     "try: argument must be (arg, (f1, f2, f3, ..., fn))"
//                                 )
//                                 .into());
//                             }
//                         }
//                     }
//                     // check for unused functions
//                     if !allow_unused_functions {
//                         for (functions_posibility_index, functions_possibility) in
//                             used_functions_and_errors.into_iter().enumerate()
//                         {
//                             for (func_index, func_possibilities) in
//                                 functions_possibility.into_iter().enumerate()
//                             {
//                                 for (func_possibility_index, errors_if_unused) in
//                                     func_possibilities.into_iter().enumerate()
//                                 {
//                                     if let Some(errs) = errors_if_unused {
//                                         let mut e = CheckError::new().msg(format!("try: For the argument {t}:\nFunction #{}{} is never used. (use `try_allow_unused` to avoid this error){}",
//                                             func_index + 1,
//                                             if functions_posibility_index != 0 || func_possibility_index != 0 {
//                                                 format!(" (func-tuple possibility {}, function possibility {})", functions_posibility_index + 1, func_possibility_index + 1)
//                                             } else {
//                                                 format!("")
//                                             },
//                                             if errs.is_empty() { "" } else { " Errors:" }));
//                                         for err in errs {
//                                             e = e.err(err);
//                                         }
//                                         return Err(e);
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 } else {
//                     return Err(format!("cannot use try with non-tuple argument").into());
//                 }
//             }
//             Ok(out)
//         }),
//         run: Arc::new(|a, _i| {
//             let tuple = a.get();
//             let tuple = tuple
//                 .as_any()
//                 .downcast_ref::<data::tuple::Tuple>()
//                 .expect("try: not a tuple");
//             let arg = &tuple.0[0];
//             let funcs = tuple.0[1].get();
//             let funcs = funcs.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
//             for func in funcs.0.iter() {
//                 let func = func.get();
//                 let func = func
//                     .as_any()
//                     .downcast_ref::<data::function::Function>()
//                     .unwrap();
//                 if func.check(&arg.get().as_type()).is_ok() {
//                     return func.run(arg.clone());
//                 }
//             }
//             unreacha ble!("try: no function found")
//         }),
//         inner_statements: None,
//     })
// }
