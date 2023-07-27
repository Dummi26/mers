use crate::lang::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some("
// Error handling in mers is inspired by Rust. Errors aren't a language feature,
// they are just a value like any other. Usually, errors have the type Err(string) or Err(SomeOtherEnum(string)),
// but this is still just a value in an enum.
// Because of the type system in mers, errors can just be used and don't need any special language features.
// This part of the mers-tutor isn't as much about error handling as it is about dealing with multiple types when you only want some of them, not all,
// but since error handling is the main use-case for this, it felt like the better title for this section.

// 1. [t]/[]
// This acts like null/nil in most languages or Option<T> in rust.
// This type indicates either '[]', a tuple of length 0 and therefore no data (null/nil/None)
//                        or '[t]', a tuple of length 1 - one value. This has to be [t] and not just t because t might be [], which would otherwise cause ambiguity.

// The type [t]/[] is returned by get(), a function that retrieves an item from a list and returns [] if the index was less than 0 or too big for the given list:
list = [1 2 3 4 5 ...]
first = list.get(0) // = [1]
second = list.get(1) // = [2]
does_not_exist = list.get(9) // = []

// To handle the result from get(), we can switch on the type:
switch! first {
  [int] \"First element in the list: {0}\".format(first.0.to_string())
  [] \"List was empty!\"
}

// If we already know that the list isn't empty, we can use assume1(). This function takes a [t]/[] and returns t. If it gets called with [], it will crash your program.
\"First element in the list: {0}\".format(first.assume1().to_string())

// 2. t/Err(e)
// This acts like Rust's Result<T, E> and is used in error-handling.
// This is mainly used by functions that do I/O (fs_* and run_command) and can also be handeled using switch or switch! statements.
// Use switch! or .debug() to see the types returned by these functions in detail.
// If switching is too much effort for you and you would like to just crash the program on any error,
// you can use assume_no_enum() to ignore all enum types:
// - t/Err(e) becomes t
// - int/float/string/Err(e)/Err(a) becomes int/float/string

// To return to the menu, change the index in list.get() so that it returns a value of type [int] instead of [].
list.get(8)
"));
    loop {
        match tutor.let_user_make_change().run(vec![]).inner_cloned() {
            VDataEnum::Tuple(v) if !v.is_empty() => {
                break;
            }
            other => {
                tutor.set_status(format!(
                    " - Returned {other} instead of a value of type [int]."
                ));
                tutor.update(None);
            }
        }
    }
}
