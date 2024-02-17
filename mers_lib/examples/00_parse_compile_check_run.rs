use std::sync::Arc;

use mers_lib::{
    data::{Data, MersType, Type},
    errors::CheckError,
    prelude_compile::{parse, CompInfo, Config, Source},
};

fn main() {
    show("1.sum(2)".to_owned());
    show("1.sum(2).println".to_owned());
    show("1.sum(2.5)".to_owned());
    show("if true { 1 } else { 0.5 }".to_owned());
}

/// Tries to parse, compile, check and run `src`,
/// then prints an error or the returned value and output type to stderr.
/// Note: The output type is not the type of the value but the one determined by `.check()` before the code even runs.
fn show(src: String) {
    eprintln!(
        "-{}",
        " -".repeat(src.lines().map(|l| l.len()).max().unwrap_or(0) / 2)
    );
    eprintln!("{src}");
    match parse_compile_check_run(src) {
        Err(e) => eprintln!("{e}"),
        Ok((t, v)) => eprintln!("Returned `{}` :: `{t}`", v.get()),
    }
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
