use crate::data::{self, Data, MersDataWInfo, Type};

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
        self.add_var("trim", func(|v: &str, _| Ok(v.trim().to_owned())))
            .add_var(
                "index_of",
                func(|(v, p): (&str, &str), _| Ok(OneOrNone(v.find(p).map(|v| v as isize)))),
            )
            .add_var(
                "index_of_rev",
                func(|(v, p): (&str, &str), _| Ok(OneOrNone(v.rfind(p).map(|v| v as isize)))),
            )
            .add_var(
                "starts_with",
                func(|(v, p): (&str, &str), _| Ok(v.starts_with(p))),
            )
            .add_var(
                "ends_with",
                func(|(v, p): (&str, &str), _| Ok(v.ends_with(p))),
            )
            .add_var(
                "str_split_once",
                func(|(v, p): (&str, &str), _| {
                    Ok(AnyOrNone(
                        v.split_once(p).map(|(a, b)| (a.to_owned(), b.to_owned())),
                    ))
                }),
            )
            .add_var(
                "str_split_once_rev",
                func(|(v, p): (&str, &str), _| {
                    Ok(AnyOrNone(
                        v.rsplit_once(p).map(|(a, b)| (a.to_owned(), b.to_owned())),
                    ))
                }),
            )
            .add_var(
                "str_split",
                func(|(v, p): (&str, &str), _| Ok(IterToList(v.split(p).map(|v| v.to_owned())))),
            )
            .add_var(
                "concat",
                util::to_mers_func(
                    |a, i| {
                        if a.iterable().is_some() {
                            Ok(Type::new(data::string::StringT))
                        } else {
                            Err(
                                format!("concat called on non-iterable type {}", a.with_info(i))
                                    .into(),
                            )
                        }
                    },
                    |a, i| {
                        Ok(Data::new(data::string::String(
                            a.get()
                                .iterable()
                                .unwrap()
                                .map(|v| v.map(|v| v.get().with_info(i).to_string()))
                                .collect::<Result<_, _>>()?,
                        )))
                    },
                ),
            )
            .add_var(
                "to_string",
                util::to_mers_func(
                    |_a, _| Ok(Type::new(data::string::StringT)),
                    |a, i| {
                        Ok(Data::new(data::string::String(
                            a.get().with_info(i).to_string(),
                        )))
                    },
                ),
            )
            .add_var(
                "substring",
                func(|v: OneOf<(&str, isize), (&str, isize, isize)>, _| {
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
                }),
            )
    }
}
