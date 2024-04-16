use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    info::Info,
};

pub fn to_mers_func(
    out: impl Fn(&Type) -> Result<Type, CheckError> + Send + Sync + 'static,
    run: impl Fn(Data) -> Data + Send + Sync + 'static,
) -> data::function::Function {
    data::function::Function {
        info: Arc::new(Info::neverused()),
        info_check: Arc::new(Mutex::new(Info::neverused())),
        out: Arc::new(move |a, _| out(a)),
        run: Arc::new(move |a, _| run(a)),
        inner_statements: None,
    }
}

pub fn to_mers_func_with_in_type(
    in_type: Type,
    out: impl Fn(&Type) -> Result<Type, CheckError> + Send + Sync + 'static,
    run: impl Fn(Data) -> Data + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func(
        move |a| {
            if a.is_included_in(&in_type) {
                out(a)
            } else {
                Err(format!("Function argument must be {in_type}, but was {a}.").into())
            }
        },
        run,
    )
}

pub fn to_mers_func_with_in_out_types(
    in_type: Type,
    out_type: Type,
    run: impl Fn(Data) -> Data + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func(
        move |a| {
            if a.is_included_in(&in_type) {
                Ok(out_type.clone())
            } else {
                Err(format!("Function argument must be {in_type}, but was {a}.").into())
            }
        },
        run,
    )
}

pub fn to_mers_func_concrete_string_to_any(
    out_type: Type,
    f: impl Fn(&str) -> Data + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_with_in_out_types(Type::new(data::string::StringT), out_type, move |a| {
        f(a.get()
            .as_any()
            .downcast_ref::<data::string::String>()
            .unwrap()
            .0
            .as_str())
    })
}
pub fn to_mers_func_concrete_string_to_string(
    f: impl Fn(&str) -> String + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_concrete_string_to_any(Type::new(data::string::StringT), move |a| {
        Data::new(data::string::String(f(a)))
    })
}

pub fn to_mers_func_concrete_string_string_to_any(
    out_type: Type,
    f: impl Fn(&str, &str) -> Data + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_with_in_out_types(
        Type::new(data::tuple::TupleT(vec![
            Type::new(data::string::StringT),
            Type::new(data::string::StringT),
        ])),
        out_type,
        move |a| {
            let a = a.get();
            let a = &a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap().0;
            let l = a[0].get();
            let r = a[1].get();
            f(
                l.as_any()
                    .downcast_ref::<data::string::String>()
                    .unwrap()
                    .0
                    .as_str(),
                r.as_any()
                    .downcast_ref::<data::string::String>()
                    .unwrap()
                    .0
                    .as_str(),
            )
        },
    )
}
pub fn to_mers_func_concrete_string_string_to_opt_int(
    f: impl Fn(&str, &str) -> Option<isize> + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_concrete_string_string_to_any(
        Type::newm(vec![
            Arc::new(data::tuple::TupleT(vec![])),
            Arc::new(data::int::IntT),
        ]),
        move |a, b| {
            f(a, b)
                .map(|v| Data::new(data::int::Int(v)))
                .unwrap_or_else(|| Data::empty_tuple())
        },
    )
}
pub fn to_mers_func_concrete_string_string_to_bool(
    f: impl Fn(&str, &str) -> bool + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_concrete_string_string_to_any(Type::new(data::bool::BoolT), move |a, b| {
        Data::new(data::bool::Bool(f(a, b)))
    })
}
pub fn to_mers_func_concrete_string_string_to_opt_string_string(
    f: impl Fn(&str, &str) -> Option<(String, String)> + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_concrete_string_string_to_any(
        Type::newm(vec![
            Arc::new(data::tuple::TupleT(vec![])),
            Arc::new(data::tuple::TupleT(vec![
                Type::new(data::string::StringT),
                Type::new(data::string::StringT),
            ])),
        ]),
        move |a, b| {
            f(a, b)
                .map(|(a, b)| {
                    Data::new(data::tuple::Tuple(vec![
                        Data::new(data::string::String(a)),
                        Data::new(data::string::String(b)),
                    ]))
                })
                .unwrap_or_else(|| Data::empty_tuple())
        },
    )
}
