use std::sync::Arc;

use crate::{
    data::{self, Data, Type},
    info::Local,
    program,
};

mod with_command_running;
mod with_iters;
mod with_list;

/// Usage: create an empty Config using Config::new(), use the methods to customize it, then get the Infos using Config::infos()
/// bundle_* for bundles (combines multiple groups or even bundles)
/// with_* for usage-oriented groups
/// add_* to add custom things
///
/// For doc-comments:
/// Description
/// `bundle_std()`
/// `type` - description
/// `var: type` - description
pub struct Config {
    globals: usize,
    info_parsed: super::parsed::Info,
    info_run: super::run::Info,
}

impl Config {
    pub fn new() -> Self {
        Self {
            globals: 0,
            info_parsed: Default::default(),
            info_run: Default::default(),
        }
    }

    /// standard utilitis used in many programs
    /// `bundle_base()`
    /// `with_list()`
    /// `with_command_running()`
    pub fn bundle_std(self) -> Self {
        self.with_command_running().with_list().bundle_base()
    }
    /// base utilities used in most programs
    /// `with_prints()`
    /// `with_math()`
    /// `with_get()`
    /// `with_iters()`
    pub fn bundle_base(self) -> Self {
        self.with_iters().with_get().with_math().with_prints()
    }

    /// `println: fn` prints to stdout and adds a newline to the end
    /// `print: fn` prints to stdout
    /// `eprintln: fn` prints to stderr and adds a newline to the end
    /// `eprint: fn` prints to stderr
    /// `debug: fn` debug-prints any value
    pub fn with_prints(self) -> Self {
        self.add_var(
            "debug".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    eprintln!("{:#?}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "eprint".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    eprint!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "eprintln".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    eprintln!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "print".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    print!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "println".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    println!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
    }
    /// `sum: fn` returns the sum of all the numbers in the tuple
    pub fn with_math(self) -> Self {
        self.add_var(
            "sum".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        let mut sumi = 0;
                        let mut sumf = 0.0;
                        let mut usef = false;
                        for val in &tuple.0 {
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
    /// `get: fn` is used to retrieve elements from collections
    pub fn with_get(self) -> Self {
        self.add_var(
            "get".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    if let Some(tuple) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(v), Some(i)) = (tuple.get(0), tuple.get(1)) {
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
                            unreachable!("get called on tuple with len < 2")
                        }
                    } else {
                        unreachable!("get called on non-tuple, arg must be (_, index)")
                    }
                }),
            }),
        )
    }

    pub fn add_var(mut self, name: String, val: Data) -> Self {
        self.info_parsed.scopes[0].init_var(name, (0, self.globals));
        self.info_run.scopes[0].init_var(self.globals, val);
        self.globals += 1;
        self
    }
    pub fn add_type(mut self, name: String, t: Type) -> Self {
        // TODO! needed for type syntax in the parser, everything else probably(?) works already
        self
    }

    pub fn infos(self) -> (super::parsed::Info, super::run::Info) {
        (self.info_parsed, self.info_run)
    }
}
