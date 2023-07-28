use std::sync::Arc;

use crate::{
    data::{self, Data},
    program,
};

use super::Config;

impl Config {
    /// `get: fn` is used to retrieve elements from collections
    pub fn with_get(self) -> Self {
        self.add_var(
            "get".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    if let (Some(v), Some(i)) = (a.get(0), a.get(1)) {
                        if let Some(i) = i.get().as_any().downcast_ref::<data::int::Int>() {
                            if let Ok(i) = i.0.try_into() {
                                if let Some(v) = v.get().get(i) {
                                    Data::one_tuple(v)
                                } else {
                                    Data::empty_tuple()
                                }
                            } else {
                                Data::empty_tuple()
                            }
                        } else {
                            unreachable!("get called with non-int index")
                        }
                    } else {
                        unreachable!("get called with less than 2 args")
                    }
                }),
            }),
        )
    }
}
