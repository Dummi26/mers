use std::sync::{Arc, Mutex};

use crate::{
    data::{Data, Type},
    info::Local,
};

mod with_base;
mod with_command_running;
mod with_get;
mod with_iters;
mod with_list;
mod with_math;
mod with_multithreading;
mod with_stdio;

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
    /// `with_command_running()`
    /// `with_multithreading()`
    pub fn bundle_std(self) -> Self {
        self.with_multithreading()
            .with_command_running()
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
        Self {
            globals: 0,
            info_parsed: Default::default(),
            info_run: Default::default(),
            info_check: Default::default(),
        }
    }

    pub fn add_var(mut self, name: String, val: Data) -> Self {
        let t = val.get().as_type();
        self.info_parsed.scopes[0].init_var(name, (0, self.globals));
        self.info_run.scopes[0].init_var(self.globals, Arc::new(Mutex::new(val)));
        self.info_check.scopes[0].init_var(self.globals, t);
        self.globals += 1;
        self
    }
    pub fn add_type(self, _name: String, _t: Type) -> Self {
        // TODO! needed for type syntax in the parser, everything else probably(?) works already
        self
    }

    pub fn infos(self) -> (super::parsed::Info, super::run::Info, super::run::CheckInfo) {
        (self.info_parsed, self.info_run, self.info_check)
    }
}
