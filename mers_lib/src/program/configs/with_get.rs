use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// `get: fn` is used to retrieve elements from collections
    pub fn with_get(self) -> Self {
        self.add_var(
            "get",
            data::function::Function {
                info: program::run::Info::neverused(),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Ok(Arc::new(|a, _i| {
                    let mut out = Type::empty();
                    for a in a.types.iter() {
                        if let Some(t) = a.as_any().downcast_ref::<data::tuple::TupleT>() {
                            if t.0.len() != 2 {
                                return Err(format!("called get on tuple with len != 2").into());
                            }
                            if !t.0[1].is_included_in_single(&data::int::IntT) {
                                return Err(format!(
                                    "called get with non-int index of type {}",
                                    t.0[1]
                                )
                                .into());
                            }
                            if let Some(v) = t.0[0].get() {
                                out.add_all(&v);
                            } else {
                                return Err(format!(
                                    "called get on non-gettable type {t}, part of {a}"
                                )
                                .into());
                            }
                        } else {
                            return Err(format!("called get on non-tuple type {a}").into());
                        }
                    }
                    Ok(Type::newm(vec![
                        Arc::new(data::tuple::TupleT(vec![out])),
                        Arc::new(data::tuple::TupleT(vec![])),
                    ]))
                })),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    if let (Some(v), Some(i)) = (a.get(0), a.get(1)) {
                        let (v, i2) = (v?, i?);
                        let o = if let Some(i3) = i2.get().as_any().downcast_ref::<data::int::Int>()
                        {
                            if let Ok(i) = i3.0.try_into() {
                                if let Some(v) = v.get().get(i) {
                                    Ok(Data::one_tuple(v?))
                                } else {
                                    Ok(Data::empty_tuple())
                                }
                            } else {
                                Ok(Data::empty_tuple())
                            }
                        } else {
                            Err("get called with non-int index".into())
                        };
                        o
                    } else {
                        Err("get called with less than 2 args".into())
                    }
                }),
                inner_statements: None,
            },
        )
    }
}
