pub mod data;
pub mod info;
pub mod parsing;
pub mod program;

pub mod prelude_compile {
    pub use crate::parsing::parse;
    pub use crate::parsing::Source;
    pub use crate::program::configs::Config;
    pub use crate::program::parsed::MersStatement as ParsedMersStatement;
    pub use crate::program::run::MersStatement as RunMersStatement;
}
