use std::sync::Arc;

use mers_lib::{
    data::{self, Data, Type},
    errors::CheckError,
    prelude_compile::{parse, Config, Source},
    program::parsed::CompInfo,
};

fn main() -> Result<(), CheckError> {
    let (_, func, info) = parse_compile_check_run(
        // The `[(String -> String)]` type annotation ensures that decorate.mers returns a `String -> String` function.
        "[(String -> String)] #include \"examples/decorate.mers\"".to_owned(),
    )?;

    // We can unwrap the downcasts because mers has type-checked that `func` is a `(String -> String)`.

    let func = func.get();
    let func = func
        .as_any()
        .downcast_ref::<data::function::Function>()
        .unwrap();

    // use the function to decorate these 3 test strings
    for input in ["my test string", "Main Menu", "O.o"] {
        let result = func.run_immut(
            Data::new(data::string::String(input.to_owned())),
            info.global.clone(),
        )?;
        let result = result.get();
        let result = &result
            .as_any()
            .downcast_ref::<data::string::String>()
            .unwrap()
            .0;
        eprintln!("{result}");
    }

    Ok(())
}

/// example 00
fn parse_compile_check_run(
    src: String,
) -> Result<(Type, Data, mers_lib::program::run::Info), CheckError> {
    let mut source = Source::new_from_string(src);
    let srca = Arc::new(source.clone());
    let parsed = parse(&mut source, &srca)?;
    let (mut i1, mut i2, mut i3) = Config::new().bundle_std().infos();
    let compiled = parsed.compile(&mut i1, CompInfo::default())?;
    let output_type = compiled.check(&mut i3, None)?;
    let output_value = compiled.run(&mut i2)?;
    Ok((output_type, output_value, i2))
}
