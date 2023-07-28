use std::sync::Arc;

use crate::{
    data::{self, Data},
    program,
};

use super::Config;

impl Config {
    /// `sum: fn` returns the sum of all the numbers in the tuple
    pub fn with_math(self) -> Self {
        self.add_var(
            "sum".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
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
