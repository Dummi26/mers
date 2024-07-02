use crate::data::{self, Data, Type};

use super::{
    gen::{function::func, AnyOrNone, IterToList, OneOf, OneOrNone},
    util, Config,
};

impl Config {
    /// `trim: fn` removes leading and trailing whitespace from a string
    /// `substring: fn` extracts part of a string. usage: (str, start).substring or (str, start, end).substring. start and end may be negative, in which case they become str.len - n: (str, 0, -1) shortens the string by 1.
    /// `index_of: fn` finds the index of a pattern in a string
    /// `index_of_rev: fn` finds the last index of a pattern in a string
    /// `starts_with: fn` checks if the string starts with the pattern
    /// `ends_with: fn` checks if the string ends with the pattern
    /// `str_split_once: fn` splits the string at the given pattern, removing that pattern from the string.
    /// `str_split_once_rev: fn` like split_str_once, but splits at the last found instance of the pattern instead of the first.
    /// `str_split: fn` splits the string at the given pattern, removing that pattern from the string.
    /// `to_string: fn` turns any argument into a (more or less useful) string representation
    /// `concat: fn` concatenates all arguments given to it. arg must be an enumerable
    pub fn with_string(self) -> Self {
        self.add_var(
            "trim".to_string(),
            Data::new(func(|v: &str| Ok(v.trim().to_owned()))),
        )
        // .add_var("index_of".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_opt_int(|v, p| v.find(p).map(|v| v as _))))
        .add_var(
            "index_of".to_string(),
            Data::new(func(|(v, p): (&str, &str)| {
                Ok(OneOrNone(v.find(p).map(|v| v as isize)))
            })),
        )
        // .add_var("index_of_rev".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_opt_int(|v, p| v.rfind(p).map(|v| v as _) )))
        .add_var(
            "index_of_rev".to_string(),
            Data::new(func(|(v, p): (&str, &str)| {
                Ok(OneOrNone(v.rfind(p).map(|v| v as isize)))
            })),
        )
        // .add_var("starts_with".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_bool(|v, p| v.starts_with(p))))
        .add_var(
            "starts_with".to_string(),
            Data::new(func(|(v, p): (&str, &str)| Ok(v.starts_with(p)))),
        )
        // .add_var("ends_with".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_bool(|v, p| v.ends_with(p))))
        .add_var(
            "ends_with".to_string(),
            Data::new(func(|(v, p): (&str, &str)| Ok(v.ends_with(p)))),
        )
        // .add_var("str_split_once".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_opt_string_string(|v, p| v.split_once(p).map(|(a, b)| (a.to_owned(), b.to_owned())))))
        .add_var(
            "str_split_once".to_string(),
            Data::new(func(|(v, p): (&str, &str)| {
                Ok(AnyOrNone(
                    v.split_once(p).map(|(a, b)| (a.to_owned(), b.to_owned())),
                ))
            })),
        )
        // .add_var("str_split_once_rev".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_opt_string_string(|v, p| v.rsplit_once(p).map(|(a, b)| (a.to_owned(), b.to_owned())))))
        .add_var(
            "str_split_once_rev".to_string(),
            Data::new(func(|(v, p): (&str, &str)| {
                Ok(AnyOrNone(
                    v.rsplit_once(p).map(|(a, b)| (a.to_owned(), b.to_owned())),
                ))
            })),
        )
        .add_var(
            "str_split".to_string(),
            Data::new(func(|(v, p): (&str, &str)| {
                Ok(IterToList(v.split(p).map(|v| v.to_owned())))
            })),
        )
        // .add_var("str_split".to_string(), Data::new(util::to_mers_func_concrete_string_string_to_any(Type::new(super::with_list::ListT(Type::new(data::string::StringT))), |v, p| Ok(Data::new(super::with_list::List(v.split(p).map(|v| Arc::new(RwLock::new(Data::new(data::string::String(v.to_owned()))))).collect()))))))
        .add_var(
            "concat".to_string(),
            Data::new(util::to_mers_func(
                |a| {
                    if a.iterable().is_some() {
                        Ok(Type::new(data::string::StringT))
                    } else {
                        Err(format!("concat called on non-iterable type {a}").into())
                    }
                },
                |a| {
                    Ok(Data::new(data::string::String(
                        a.get()
                            .iterable()
                            .unwrap()
                            .map(|v| v.map(|v| v.get().to_string()))
                            .collect::<Result<_, _>>()?,
                    )))
                },
            )),
        )
        .add_var(
            "to_string".to_string(),
            Data::new(util::to_mers_func(
                |_a| Ok(Type::new(data::string::StringT)),
                |a| Ok(Data::new(data::string::String(a.get().to_string()))),
            )),
        )
        // .add_var("substring".to_string(), Data::new(util::to_mers_func(
        //     |a| {
        //         for t in a.types.iter() {
        //             if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
        //                 if t.0.len() != 2 && t.0.len() != 3 {
        //                     return Err(format!("cannot call substring with tuple argument of len != 3").into());
        //                 }
        //                 if !t.0[0].is_included_in_single(&data::string::StringT) {
        //                     return Err(format!("cannot call substring with tuple argument that isn't (*string*, int, int)").into());
        //                 }
        //                 if !t.0[1].is_included_in_single(&data::int::IntT) {
        //                     return Err(format!("cannot call substring with tuple argument that isn't (string, *int*, int)").into());
        //                 }
        //                 if t.0.len() > 2 && !t.0[2].is_included_in_single(&data::int::IntT) {
        //                     return Err(format!("cannot call substring with tuple argument that isn't (string, int, *int*)").into());
        //                 }
        //             } else {
        //                 return Err(format!("cannot call substring with non-tuple argument.").into());
        //             }
        //         }
        //         Ok(if a.types.is_empty() {
        //             Type::empty()
        //         } else {
        //             Type::new(data::string::StringT)
        //         })
        //     },
        //     |a| {
        //         let tuple = a.get();
        //         let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().expect("called substring with non-tuple arg");
        //         let (s, start, end) = (&tuple.0[0], &tuple.0[1], tuple.0.get(2));
        //         let s = s.get();
        //         let s = &s.as_any().downcast_ref::<data::string::String>().unwrap().0;
        //         let start = start.get();
        //         let start = start.as_any().downcast_ref::<data::int::Int>().unwrap().0;
        //         let start = if start < 0 { s.len().saturating_sub(start.abs() as usize) } else { start as usize };
        //         let end = end
        //             .map(|end| end.get())
        //             .map(|end| end.as_any().downcast_ref::<data::int::Int>().unwrap().0)
        //             .map(|i| if i < 0 { s.len().saturating_sub(i.abs() as usize) } else { i as usize })
        //             .unwrap_or(usize::MAX);
        //         let end = end.min(s.len());
        //         if end < start {
        //             return Ok(Data::new(data::string::String(String::new())));
        //         }
        //         Ok(Data::new(data::string::String(s[start..end].to_owned())))
        //     })
        // ))
        .add_var(
            "substring".to_string(),
            Data::new(func(|v: OneOf<(&str, isize), (&str, isize, isize)>| {
                let (s, start, end) = match v {
                    OneOf::A((t, s)) => (t, s, None),
                    OneOf::B((t, s, e)) => (t, s, Some(e)),
                };
                let start = if start < 0 {
                    s.len().saturating_sub(start.abs() as usize)
                } else {
                    start as usize
                };
                let end = end
                    .map(|i| {
                        if i < 0 {
                            s.len().saturating_sub(i.abs() as usize)
                        } else {
                            i as usize
                        }
                    })
                    .unwrap_or(usize::MAX);
                let end = end.min(s.len());
                if end < start {
                    return Ok(String::new());
                }
                Ok(s[start..end].to_owned())
            })),
        )
    }
}
