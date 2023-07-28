/// data and types in mers
pub mod data;
/// shared code handling scopes to guarantee that compiler and runtime scopes match
pub mod info;
/// parser implementation.
#[cfg(feature = "parse")]
pub mod parsing;
pub mod program;

#[cfg(feature = "parse")]
pub mod prelude_compile {
    pub use crate::parsing::parse;
    pub use crate::parsing::Source;
    pub use crate::program::configs::Config;
    pub use crate::program::parsed::MersStatement as ParsedMersStatement;
    pub use crate::program::run::MersStatement as RunMersStatement;
}

/// can be used to extend the mers config.
/// with this, you can add values (usually functions),
/// or add your own types to the language:
///
///     fn add_thing(cfg: Config) -> Config {
///         /// use the methods on Config to add things (see the Config source code for examples)
///     }
///
/// then use the Config when compiling and running your code, and your customizations will be available.
pub mod prelude_extend_config {
    pub use crate::program::configs::Config;
}
