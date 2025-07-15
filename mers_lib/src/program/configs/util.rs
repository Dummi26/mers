use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    info::Info,
};

pub fn to_mers_func(
    out: impl Fn(&Type, &mut crate::program::run::CheckInfo) -> Result<Type, CheckError>
        + Send
        + Sync
        + 'static,
    run: impl Fn(Data, &mut crate::program::run::Info) -> Result<Data, CheckError>
        + Send
        + Sync
        + 'static,
) -> data::function::Function {
    data::function::Function {
        info: Info::neverused(),
        info_check: Arc::new(Mutex::new(Info::neverused())),
        fixed_type: None,
        fixed_type_out: Arc::new(Mutex::new(None)),
        out: Ok(Arc::new(move |a, i| out(a, i))),
        run: Arc::new(move |a, i| run(a, i)),
        inner_statements: None,
    }
}

pub fn to_mers_func_with_in_type(
    in_type: Type,
    out: impl Fn(&Type, &mut crate::program::run::CheckInfo) -> Result<Type, CheckError>
        + Send
        + Sync
        + 'static,
    run: impl Fn(Data, &mut crate::program::run::Info) -> Result<Data, CheckError>
        + Send
        + Sync
        + 'static,
) -> data::function::Function {
    to_mers_func(
        move |a, i| {
            if a.is_included_in(&in_type) {
                out(a, i)
            } else {
                Err(format!(
                    "Function argument must be {}, but was {}.",
                    in_type.with_info(i),
                    a.with_info(i)
                )
                .into())
            }
        },
        run,
    )
}

pub fn to_mers_func_with_in_out_types(
    in_type: Type,
    out_type: Type,
    run: impl Fn(Data, &mut crate::program::run::Info) -> Result<Data, CheckError>
        + Send
        + Sync
        + 'static,
) -> data::function::Function {
    data::function::Function::new_static(vec![(in_type, out_type)], move |a, i| run(a, i))
}

pub fn to_mers_func_concrete_string_to_any(
    out_type: Type,
    f: impl Fn(&str) -> Result<Data, CheckError> + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_with_in_out_types(Type::new(data::string::StringT), out_type, move |a, _| {
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
        Ok(Data::new(data::string::String(f(a))))
    })
}

pub fn to_mers_func_concrete_string_string_to_any(
    out_type: Type,
    f: impl Fn(&str, &str) -> Result<Data, CheckError> + Send + Sync + 'static,
) -> data::function::Function {
    to_mers_func_with_in_out_types(
        Type::new(data::tuple::TupleT(vec![
            Type::new(data::string::StringT),
            Type::new(data::string::StringT),
        ])),
        out_type,
        move |a, _| {
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
