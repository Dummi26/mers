use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, MersType, Type},
    program::run::{CheckInfo, Info},
};

use super::Config;

impl Config {
    /// `trim: fn` removes leading and trailing whitespace from a string
    /// `substring: fn` extracts part of a string. usage: (str, start).substring or (str, start, end).substring. start and end may be negative, in which case they become str.len - n: (str, 0, -1) shortens the string by 1.
    /// `index_of: fn` finds the index of a pattern in a string
    /// `index_of_rev: fn` finds the last index of a pattern in a string
    /// `to_string: fn` turns any argument into a (more or less useful) string representation
    /// `concat: fn` concatenates all arguments given to it. arg must be an enumerable
    pub fn with_string(self) -> Self {
        self.add_var("trim".to_string(), Data::new(data::function::Function {
            info: Arc::new(Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.is_included_in(&data::string::StringT) {
                Ok(Type::new(data::string::StringT))
            } else {
                Err(format!("cannot call trim on non-strings").into())
            }),
            run: Arc::new(|a, _i| {
                Data::new(data::string::String(a.get().as_any().downcast_ref::<data::string::String>().unwrap().0.trim().to_owned()))
            }),
                inner_statements: None,
        })).add_var("concat".to_string(), Data::new(data::function::Function {
            info: Arc::new(Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.iterable().is_some() {
                Ok(Type::new(data::string::StringT))
            } else {
                Err(format!("concat called on non-iterable type {a}").into())
            }),
            run: Arc::new(|a, _i| Data::new(data::string::String(a.get().iterable().unwrap().map(|v| v.get().to_string()).collect()))),
                inner_statements: None,
        })).add_var("to_string".to_string(), Data::new(data::function::Function {
            info: Arc::new(Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|_a, _i| Ok(Type::new(data::string::StringT))),
            run: Arc::new(|a, _i| Data::new(data::string::String(a.get().to_string()))),
                inner_statements: None,
        })).add_var("index_of".to_string(), Data::new(data::function::Function {
            info: Arc::new(Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.is_included_in(&data::tuple::TupleT(vec![Type::new(data::string::StringT), Type::new(data::string::StringT)])) {
                Ok(Type::newm(vec![
                    Arc::new(data::tuple::TupleT(vec![])),
                    Arc::new(data::int::IntT),
                ]))
            } else {
                Err(format!("wrong args for index_of: must be (string, string)").into())
            }),
            run: Arc::new(|a, _i| index_of(a, false)),
                inner_statements: None,
        })).add_var("index_of_rev".to_string(), Data::new(data::function::Function {
            info: Arc::new(Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| if a.is_included_in(&data::tuple::TupleT(vec![Type::new(data::string::StringT), Type::new(data::string::StringT)])) {
                Ok(Type::newm(vec![
                    Arc::new(data::tuple::TupleT(vec![])),
                    Arc::new(data::int::IntT),
                ]))
            } else {
                Err(format!("wrong args for index_of: must be (string, string)").into())
            }),
            run: Arc::new(|a, _i| index_of(a, true)),
                inner_statements: None,
        })).add_var("substring".to_string(), Data::new(data::function::Function {
            info: Arc::new(Info::neverused()),
            info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            out: Arc::new(|a, _i| {
                for t in a.types.iter() {
                    if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                        if t.0.len() != 2 && t.0.len() != 3 {
                            return Err(format!("cannot call substring with tuple argument of len != 3").into());
                        }
                        if !t.0[0].is_included_in(&data::string::StringT) {
                            return Err(format!("cannot call substring with tuple argument that isn't (*string*, int, int)").into());
                        }
                        if !t.0[1].is_included_in(&data::int::IntT) {
                            return Err(format!("cannot call substring with tuple argument that isn't (string, *int*, int)").into());
                        }
                        if t.0.len() > 2 && !t.0[2].is_included_in(&data::int::IntT) {
                            return Err(format!("cannot call substring with tuple argument that isn't (string, int, *int*)").into());
                        }
                    } else {
                        return Err(format!("cannot call substring with non-tuple argument.").into());
                    }
                }
                Ok(if a.types.is_empty() {
                    Type::empty()
                } else {
                    Type::new(data::string::StringT)
                })
            }),
            run: Arc::new(|a, _i| {
                let tuple = a.get();
                let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().expect("called substring with non-tuple arg");
                let (s, start, end) = (&tuple.0[0], &tuple.0[1], tuple.0.get(2));
                let s = s.get();
                let s = &s.as_any().downcast_ref::<data::string::String>().unwrap().0;
                let start = start.get();
                let start = start.as_any().downcast_ref::<data::int::Int>().unwrap().0;
                let start = if start < 0 { s.len().saturating_sub(start.abs() as usize) } else { start as usize };
                let end = end
                    .map(|end| end.get())
                    .map(|end| end.as_any().downcast_ref::<data::int::Int>().unwrap().0)
                    .map(|i| if i < 0 { s.len().saturating_sub(i.abs() as usize) } else { i as usize })
                    .unwrap_or(usize::MAX);
                let end = end.min(s.len());
                if end < start {
                    return Data::new(data::string::String(String::new()));
                }
                Data::new(data::string::String(s[start..end].to_owned()))

            }),
                inner_statements: None,
        }))
    }
}

fn index_of(a: Data, rev: bool) -> Data {
    let a = a.get();
    let a = a
        .as_any()
        .downcast_ref::<data::tuple::Tuple>()
        .expect("index_of called on non-tuple");
    let src = a.0[0].get();
    let src = &src
        .as_any()
        .downcast_ref::<data::string::String>()
        .unwrap()
        .0;
    let pat = a.0[1].get();
    let pat = &pat
        .as_any()
        .downcast_ref::<data::string::String>()
        .unwrap()
        .0;
    let i = if rev { src.rfind(pat) } else { src.find(pat) };
    if let Some(i) = i {
        Data::new(data::int::Int(i as _))
    } else {
        Data::empty_tuple()
    }
}
