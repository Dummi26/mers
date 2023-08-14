use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    program::{
        self,
        run::{CheckError, CheckInfo},
    },
};

use super::Config;

impl Config {
    /// `sum: fn` returns the sum of all the numbers in the tuple
    pub fn with_math(self) -> Self {
        self.add_var(
            "sum".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, i| {
                    let mut ints = false;
                    let mut floats = false;
                    for a in &a.types {
                        if let Some(i) = a.iterable() {
                            if i.types
                                .iter()
                                .all(|t| t.as_any().downcast_ref::<data::int::IntT>().is_some())
                            {
                                ints = true;
                            } else if i.types.iter().all(|t| {
                                t.as_any().downcast_ref::<data::int::IntT>().is_some()
                                    || t.as_any().downcast_ref::<data::float::FloatT>().is_some()
                            }) {
                                floats = true;
                            } else {
                                return Err(CheckError(format!("cannot get sum of iterator over type {i} because it contains types that aren't int/float")))
                            }
                        } else {
                            return Err(CheckError(format!(
                                "cannot get sum of non-iterable type {a}"
                            )));
                        }
                    }
                    Ok(match (ints, floats) {
                        (_, true) => Type::new(data::float::FloatT),
                        (true, false) => Type::new(data::int::IntT),
                        (false, false) => Type::empty(),
                    })
                }),
                run: Arc::new(|a, _i| {
                    if let Some(i) = a.get().iterable() {
                        let mut sumi = 0;
                        let mut sumf = 0.0;
                        let mut usef = false;
                        for val in i {
                            if let Some(i) = val.get().as_any().downcast_ref::<data::int::Int>() {
                                sumi += i.0;
                            } else if let Some(i) =
                                val.get().as_any().downcast_ref::<data::float::Float>()
                            {
                                sumf += i.0;
                                usef = true;
                            }
                        }
                        if usef {
                            Data::new(data::float::Float(sumi as f64 + sumf))
                        } else {
                            Data::new(data::int::Int(sumi))
                        }
                    } else {
                        unreachable!("sum called on non-tuple")
                    }
                }),
            }),
        )
    }
}
