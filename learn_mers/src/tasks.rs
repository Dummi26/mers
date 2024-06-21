use std::sync::{Arc, Mutex};

use mers_lib::{
    data::{self, Data, MersType, Type},
    errors::CheckError,
    prelude_compile::{RunMersStatement, Source},
    prelude_extend_config::Config,
};

use crate::Args;

pub(crate) fn tasks(
    args: &Args,
) -> Vec<(
    String,
    Box<dyn Fn() -> Config>,
    Box<
        dyn FnMut(
            Result<
                (
                    Type,
                    Box<dyn RunMersStatement>,
                    mers_lib::program::parsed::Info,
                    mers_lib::program::run::Info,
                    mers_lib::program::run::CheckInfo,
                ),
                CheckError,
            >,
            Source,
        ) -> bool,
    >,
)> {
    let file = args.file.to_string_lossy();
    vec![
        (
            format!(
                " Hello, World!
---------------
Hello! I've saved a file at {file}.
You can always use `mers run '{file}'` to actually run it, but for this introduction, just open the file in the editor of your choice.

You should see mers' version of the \"Hello, World!\" program. Your task is to cause an error - to write code that doesn't work.

When done, save the file and you should get feedback right here."
            ),
            Box::new(|| Config::new().with_stdio()),
            Box::new(|v, s| match v {
                Ok(..) => {
                    eprintln!("nope, that still compiles.");
                    false
                }
                Err(e) => {
                    eprintln!("{}", e.display(&s));
                    eprintln!("Nice! Just for fun, the error you created can be seen above.");
                    true
                }
            }),
        ),
        (
            format!(
                " Hello, Function!
------------------
Functions in mers are created using `->`:
`arg -> arg`
This is a function that does nothing, it just returns its argument.
`n -> (n, 1).sum`
This function returns `n + 1`.

Your task is to create any function."
            ),
            Box::new(|| Config::new().with_math()),
            Box::new(|v, s| match v {
                Ok((t, _, _, _, _)) => {
                    if t.types
                        .iter()
                        .all(|t| t.as_any().is::<data::function::FunctionT>())
                    {
                        eprintln!(
                            "Nice! Note that, even though you didn't see an error, your function may not have been correct. This will be explained later."
                        );
                        true
                    } else {
                        eprintln!("This expression has type {t}, which isn't a function");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("{}", e.display(&s));
                    false
                }
            }),
        ),
        (
            format!(
                " Using Functions
-----------------
Mers functions require exactly one argument.
To use a function, use the `<argument>.<function>` syntax:
`\"Hi\".println`
(`println` is a function, `\"Hi\"` is the argument)

In this task, `func` will be a function. Your task is to use it.
`func` works with any argument, so this should be quite easy."
            ),
            Box::new(|| {
                Config::new().with_stdio().add_var(
                    "func".to_string(),
                    Data::new(data::function::Function {
                        info: Arc::new(mers_lib::info::Info::neverused()),
                        info_check: Arc::new(Mutex::new(mers_lib::info::Info::neverused())),
                        out: Arc::new(|_, _| Ok(Type::empty())),
                        run: Arc::new(|_, _| Data::empty_tuple()),
                    }),
                )
            }),
            Box::new(|v, s| match v {
                Ok((t, _, _, _, _)) => {
                    if t.types.is_empty() {
                        eprintln!(
                            "Nice! You successfully achieved nothing by using a function that does nothing."
                        );
                        true
                    } else {
                        eprintln!("Hm, doesn't look like using `func` is the last thing your program does...");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("{}", e.display(&s));
                    false
                }
            }),
        ),(
            format!(
                " Hello, Variables!
------------------
To create a new variable in mers, use `:=`:
`greeting := \"Hello\"`
You can use variables by just writing their name:
`greeting.println`

In this task, I'll add the variable `im_done`.
To move on to the next task, return the value stored in it by writing `im_done` at the end of your file.
You can also store the value in another variable first:
`value := im_done`
`value`"
            ),
            Box::new(|| Config::new().add_var("im_done".to_string(), Data::one_tuple(Data::empty_tuple()))),
            Box::new(|v, s| match v {
                Ok((t, _, _, _, _)) => {
                    if t.one_tuple_content().is_some_and(|c| c.is_zero_tuple())
                    {
                        true
                    } else {
                        eprintln!("You returned a value of type {t}, which isn't the type of `im_done`.");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("{}", e.display(&s));
                    false
                }
            }),
        ),
        (
            format!(
                " Functions with multiple arguments
-----------------------------------
Mers functions only have one argument. To give a function multiple values, we have to use tuples:
`(\"Hello, \", \"World!\").concat`
(`concat` joins the two strings together, creating `\"Hello, World!\"`.)

When writing your own functions, you can destructure these tuples:
`(a, b) -> (b, a).concat`
This creates a function which can only be called with a 2-long tuple.

Your task is to assign this function to a variable `swapped`:
`swapped := (a, b) -> (b, a).concat`
Then, try to call the function with a wrong argument:
`\"hi\".swapped`
`().swapped`
`(\"a\", \"b\", \"c\").swapped`
To complete the task, use the function in a way that won't cause any errors."
            ),
            Box::new(|| Config::new().with_string()),
            Box::new(|v, s| match v {
                Ok((t, _, _, _, _)) => {
                    if t.is_included_in(&data::string::StringT) {
                        true
                    } else {
                        eprintln!("Whatever you did compiles, but doesn't seem to use the `swapped` function...");
                        false
                    }
                }
                Err(e) => {
                    eprintln!("{}", e.display(&s));
                    false
                }
            }),
        ),
    ]
}
