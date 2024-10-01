use std::sync::{Arc, RwLock};

use crate::{
    data::{
        self,
        int::{INT_MAX, INT_MIN},
        Data, MersData, Type,
    },
    errors::CheckError,
    info::Local,
    program::run::{CheckInfo, CheckLocalGlobalInfo, RunLocalGlobalInfo},
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
        let info_parsed = crate::program::parsed::Info::new(
            crate::program::parsed::LocalGlobalInfo::new(Arc::new(Default::default())),
        );
        let mut info_check = CheckInfo::new(CheckLocalGlobalInfo::new(Arc::clone(
            &info_parsed.global.object_fields,
        )));
        let info_run = crate::program::run::Info::new(RunLocalGlobalInfo::new(Arc::clone(
            &info_parsed.global.object_fields,
        )));
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
        init_d!(data::bool::TrueT);
        init_d!(data::bool::FalseT);
        info_check
            .scopes
            .last_mut()
            .unwrap()
            .types
            .insert("Bool".to_owned(), Ok(Arc::new(data::bool::bool_type())));
        init_d!(data::byte::ByteT);
        info_check.scopes.last_mut().unwrap().types.insert(
            "Int".to_owned(),
            // Ok(Arc::new(data::Type::new(data::int::INT_T_ALL))),
            Err(Arc::new(|range, _| {
                Ok(Arc::new(Type::new({
                    let range = range.trim();
                    if range.is_empty() {
                        data::int::IntT(INT_MIN, INT_MAX)
                    } else if let Some((min, max)) = range.split_once("..") {
                        let (min, max) = (min.trim(), max.trim());
                        let min = if min.is_empty() {
                            data::int::INT_MIN
                        } else if let Ok(v) = min.parse() {
                            v
                        } else {
                            return Err(CheckError::new().msg_str(format!("In type `Int<{min}..{max}>`: min was present but not a valid integer.")));
                        };
                        let max = if max.is_empty() {
                            data::int::INT_MAX
                        } else if let Ok(v) = max.parse() {
                            v
                        } else {
                            return Err(CheckError::new().msg_str(format!("In type `Int<{min}..{max}>`: max was present but not a valid integer.")));
                        };
                        if min > max {
                            return Err(CheckError::new().msg_str(format!("In type `Int<{min}..{max}>`: min ({min}) must be smaller than or equal to max ({max}). Did you mean `Int<{max}..{min}>`?")));
                        }
                        crate::data::int::IntT(min, max)
                    } else if let Ok(v) = range.parse() {
                        crate::data::int::IntT(v, v)
                    } else {
                        return Err(CheckError::new().msg_str(format!("In type `Int<{range}>`: Invalid range. Either use `Int` (or `Int<>` or `Int<..>`) for the entire integer range, `Int<n>` for a specific number, `Int<n..>` for all numbers `>=n`, `Int<..m>` for all numbers `<=m`, or `Int<n..m>` for all numbers `>=n` and `<= m`.")));
                    }
                })))
            })),
        );
        init_d!(data::float::FloatT);
        init_d!(data::string::StringT);
        Self {
            globals: 0,
            info_parsed,
            info_run,
            info_check,
        }
    }

    /// Add a variable. Its type will be that of the value stored in `val`.
    pub fn add_var(self, name: impl Into<String>, val: impl MersData) -> Self {
        let t = val.as_type();
        self.add_var_from_arc(name.into(), Arc::new(RwLock::new(Data::new(val))), t)
    }
    pub fn add_var_from_data(self, name: String, val: Data) -> Self {
        let t = val.get().as_type();
        self.add_var_from_arc(name, Arc::new(RwLock::new(val)), t)
    }
    pub fn add_var_from_arc(
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
