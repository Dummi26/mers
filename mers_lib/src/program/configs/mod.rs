use std::sync::{Arc, RwLock};

use crate::{
    data::{self, Data, MersType},
    errors::CheckError,
    info::Local,
    program::run::CheckInfo,
};

mod with_base;
mod with_command_running;
mod with_get;
mod with_iters;
mod with_list;
mod with_math;
mod with_multithreading;
mod with_stdio;
mod with_string;

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
    info_check: super::run::CheckInfo,
}

impl Config {
    /// standard utilitis used in many programs
    /// `bundle_base()`
    /// `with_stdio()`
    /// `with_list()`
    /// `with_string()`
    /// `with_command_running()`
    /// `with_multithreading()`
    pub fn bundle_std(self) -> Self {
        self.with_multithreading()
            .with_command_running()
            .with_string()
            .with_list()
            .with_stdio()
            .bundle_base()
    }
    /// base utilities used in most programs
    /// `with_base()`
    /// `with_math()`
    /// `with_get()`
    /// `with_iters()`
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
                    .insert(t.to_string(), Ok(Arc::new(t)));
            };
        }
        init_d!(data::bool::BoolT);
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

    pub fn add_var(mut self, name: String, val: Data) -> Self {
        let t = val.get().as_type();
        self.info_parsed.scopes[0].init_var(name, (0, self.globals));
        self.info_run.scopes[0].init_var(self.globals, Arc::new(RwLock::new(val)));
        self.info_check.scopes[0].init_var(self.globals, t);
        self.globals += 1;
        self
    }
    pub fn add_type(
        mut self,
        name: String,
        t: Result<
            Arc<dyn MersType>,
            Arc<dyn Fn(&str, &CheckInfo) -> Result<Arc<dyn MersType>, CheckError> + Send + Sync>,
        >,
    ) -> Self {
        self.info_check.scopes[0].types.insert(name, t);
        self
    }

    pub fn infos(self) -> (super::parsed::Info, super::run::Info, super::run::CheckInfo) {
        (self.info_parsed, self.info_run, self.info_check)
    }
}
