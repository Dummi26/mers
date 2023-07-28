use std::sync::Arc;

use crate::{
    data::{self, Data, Type},
    info::Local,
    program,
};

mod with_command_running;
mod with_get;
mod with_iters;
mod with_list;
mod with_math;
mod with_prints;

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

    pub fn new() -> Self {
        Self {
            globals: 0,
            info_parsed: Default::default(),
            info_run: Default::default(),
        }
    }

    pub fn add_var(mut self, name: String, val: Data) -> Self {
        self.info_parsed.scopes[0].init_var(name, (0, self.globals));
        self.info_run.scopes[0].init_var(self.globals, val);
        self.globals += 1;
        self
    }
    pub fn add_type(self, _name: String, _t: Type) -> Self {
        // TODO! needed for type syntax in the parser, everything else probably(?) works already
        self
    }

    pub fn infos(self) -> (super::parsed::Info, super::run::Info) {
        (self.info_parsed, self.info_run)
    }
}
