use std::sync::Arc;

use mers_lib::{
    data::{self, Data, Type},
    errors::CheckError,
    prelude_compile::{parse, Config, Source},
    program::parsed::CompInfo,
};

fn main() {
    eprintln!("This is valid:");
    run("my_custom_var.debug.rust_func.debug".to_owned()).unwrap();
    eprintln!();

    eprintln!("This is not:");
    let e = run("5.rust_func".to_owned()).err().unwrap();
    eprintln!("{e:?}");
}
fn run(src: String) -> Result<(), CheckError> {
    let mut source = Source::new_from_string(src);
    let srca = Arc::new(source.clone());
    let parsed = parse(&mut source, &srca)?;

    // Add our custom variables to the `Config`
    let (mut i1, mut i2, mut i3) = Config::new()
        .bundle_std()
        .add_var(
            "my_custom_var".to_owned(),
            Data::new(data::string::String(format!("my custom value!"))),
        )
        .add_var(
            "rust_func".to_owned(),
            Data::new(data::function::Function::new_generic(
                |arg| {
                    // If the input is a string, the output is a string.
                    // Otherwise, the function is used incorrectly.
                    if arg.is_included_in_single(&data::string::StringT) {
                        Ok(Type::new(data::string::StringT))
                    } else {
                        // Wrong argument type. The code won't compile and this is the error message shown to the user.
                        Err(format!("Can't call rust_func with non-string argument {arg}!").into())
                    }
                },
                |arg| {
                    let arg = arg.get();
                    let arg = &arg
                        .as_any()
                        .downcast_ref::<data::string::String>()
                        .unwrap()
                        .0;
                    Ok(Data::new(data::string::String(arg.chars().rev().collect())))
                },
            )),
        )
        .infos();

    let compiled = parsed.compile(&mut i1, CompInfo::default())?;
    compiled.check(&mut i3, None)?;
    compiled.run(&mut i2)?;
    Ok(())
}
