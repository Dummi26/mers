/// data and types in mers
pub mod data;
/// struct to represent errors the user may face
pub mod errors;
/// shared code handling scopes to guarantee that compiler and runtime scopes match
pub mod info;
/// parser implementation.
#[cfg(feature = "parse")]
pub mod parsing;
#[cfg(feature = "pretty-print")]
pub mod pretty_print;
pub mod program;
pub mod theme;

#[cfg(feature = "parse")]
pub mod prelude_compile {
    pub use crate::parsing::check;
    pub use crate::parsing::check_mut;
    pub use crate::parsing::compile;
    pub use crate::parsing::compile_mut;
    pub use crate::parsing::parse;
    pub use crate::parsing::Source;
    pub use crate::program::configs::Config;
}

/// can be used to extend the mers config.
/// with this, you can add values (usually functions),
/// or your own custom types to the language.
///
/// Your customizations will only be available if you use
/// the infos you got from calling `.infos()` on your `Config` after customizing it.
pub mod prelude_extend_config {
    pub use crate::program::configs::Config;
}
