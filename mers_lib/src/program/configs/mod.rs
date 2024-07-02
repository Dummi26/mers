use std::sync::{Arc, RwLock};

use crate::{
    data::{self, Data, Type},
    errors::CheckError,
    info::Local,
    program::run::CheckInfo,
};

pub mod gen;
pub mod util;
pub mod with_base;
pub mod with_command_running;
pub mod with_get;
pub mod with_iters;
pub mod with_list;
pub mod with_math;
pub mod with_multithreading;
pub mod with_stdio;
pub mod with_string;

/// Usage: create an empty Config using Config::new(), use the methods to customize it, then get the Infos using Config::infos()
/// bundle_* for bundles (combines multiple groups or even bundles)
/// with_* for usage-oriented groups
/// add_* to add custom things
pub struct Config {
    globals: usize,
    info_parsed: super::parsed::Info,
    info_run: super::run::Info,
    info_check: super::run::CheckInfo,
}

impl Config {
    /// standard utilitis used in many programs
    ///
    /// - `bundle_pure()`
    /// - `with_stdio()`
    /// - `with_command_running()`
    /// - `with_multithreading()`
    pub fn bundle_std(self) -> Self {
        self.with_multithreading()
            .with_command_running()
            .with_stdio()
            .bundle_pure()
    }
    /// standard utilities, but don't allow code to do any I/O.
    /// (multithreading can be added using `.with_multithreading()`)
    ///
    /// - `bundle_base()`
    /// - `with_list()`
    /// - `with_string()`
    pub fn bundle_pure(self) -> Self {
        self.with_string().with_list().bundle_base()
    }
    /// base utilities used in most programs
    ///
    /// - `with_base()`
    /// - `with_math()`
    /// - `with_get()`
    /// - `with_iters()`
    pub fn bundle_base(self) -> Self {
        self.with_iters().with_get().with_math().with_base()
    }

    pub fn new() -> Self {
        let mut info_check: CheckInfo = Default::default();
        macro_rules! init_d {
            ($e:expr) => {
                let t = $e;
                info_check
                    .scopes
                    .last_mut()
                    .unwrap()
                    .types
                    .insert(t.to_string(), Ok(Arc::new(data::Type::new(t))));
            };
        }
        init_d!(data::bool::BoolT);
        init_d!(data::byte::ByteT);
        init_d!(data::int::IntT);
        init_d!(data::float::FloatT);
        init_d!(data::string::StringT);
        Self {
            globals: 0,
            info_parsed: Default::default(),
            info_run: Default::default(),
            info_check,
        }
    }

    /// Add a variable. Its type will be that of the value stored in `val`.
    pub fn add_var(self, name: String, val: Data) -> Self {
        let t = val.get().as_type();
        self.add_var_arc(name, Arc::new(RwLock::new(val)), t)
    }
    pub fn add_var_arc(
        mut self,
        name: String,
        val: Arc<RwLock<Data>>,
        val_type: crate::data::Type,
    ) -> Self {
        self.info_parsed.scopes[0].init_var(name, (0, self.globals));
        self.info_run.scopes[0].init_var(self.globals, val);
        self.info_check.scopes[0].init_var(self.globals, val_type);
        self.globals += 1;
        self
    }
    pub fn add_type(
        mut self,
        name: String,
        t: Result<
            Arc<Type>,
            Arc<dyn Fn(&str, &CheckInfo) -> Result<Arc<Type>, CheckError> + Send + Sync>,
        >,
    ) -> Self {
        self.info_check.scopes[0].types.insert(name, t);
        self
    }

    pub fn infos(self) -> (super::parsed::Info, super::run::Info, super::run::CheckInfo) {
        (self.info_parsed, self.info_run, self.info_check)
    }
}
