/// data and types in mers
pub mod data;
/// struct to represent errors the user may face
pub mod errors;
/// shared code handling scopes to guarantee that compiler and runtime scopes match
pub mod info;
/// parser implementation.
#[cfg(feature = "parse")]
pub mod parsing;
pub mod program;

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
/// or add your own types to the language:
///
///     use mers_lib::prelude_extend_config::Config;
///     fn add_thing(cfg: Config) -> Config {
///         // use the methods on Config to add things (see the Config source code for examples)
///         cfg.add_var("my_var".to_owned(), todo!())
///     }
///
/// then use the Config when compiling and running your code, and your customizations will be available.
pub mod prelude_extend_config {
    pub use crate::program::configs::Config;
}

#[test]
fn test_examples() {
    for example in std::fs::read_dir("../examples").unwrap() {
        let path = example.unwrap().path();
        eprintln!("Checking file {path:?}.");
        let src = prelude_compile::Source::new_from_file(path).unwrap();
        let (mut i1, _, mut i3) = prelude_compile::Config::new().bundle_std().infos();
        prelude_compile::parse(&mut src.clone(), &std::sync::Arc::new(src))
            .unwrap()
            .compile(&mut i1, program::parsed::CompInfo::default())
            .unwrap()
            .check(&mut i3, None)
            .unwrap();
    }
}
