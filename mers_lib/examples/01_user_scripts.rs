use std::sync::Arc;

use mers_lib::{
    data::{self, Data, MersType, Type},
    errors::CheckError,
    prelude_compile::{parse, CompInfo, Config, Source},
};

fn main() -> Result<(), CheckError> {
    let (_, func) = parse_compile_check_run(
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
        let result = func.run(Data::new(data::string::String(input.to_owned())));
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

fn parse_compile_check_run(src: String) -> Result<(Type, Data), CheckError> {
    // prepare the string for parsing
    let mut source = Source::new_from_string(src);
    // this is used for error messages
    let srca = Arc::new(source.clone());
    // parse the code
    let parsed = parse(&mut source, &srca)?;
    // get infos
    let (mut i1, mut i2, mut i3) = Config::new().bundle_std().infos();
    // compile
    let compiled = parsed.compile(&mut i1, CompInfo::default())?;
    // check (this step is optional, but if it is skipped when it would have returned an error, `run` will likely panic)
    let output_type = compiled.check(&mut i3, None)?;
    // run
    let output_value = compiled.run(&mut i2);
    // check that the predicted output type was correct
    assert!(output_value.get().as_type().is_included_in(&output_type));
    // return the produced value
    Ok((output_type, output_value))
}
