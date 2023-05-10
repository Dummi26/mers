use crate::script::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some("
// Mers uses a type system to verify your programs,
// which prevents your program from crashing.
// Mers will verify that your program is valid and will not run into issues before it is executed.
// This way, errors are found when you write the program, not when you run it. If mers runs your program
// it is almost always safe to assume that it will not crash.

// for example, this will cause an error because you cannot subtract text from numbers.
//  sub(15 \"some text\")

// mers can verify this type-safety in all programs, no matter how complicated they are:
//  a = 15 // mers knows: a is an int
//  b = \"some text\" // mers knows: b is a string
//  sub(a b) // mers knows: it can't subtract a string from an int

// Just like other statically-typed languages, mers achieves this safety by assigning a certain type to each variable (technically to each statement).
// However, mers' type-system has one quirk that sets it apart from most others:

a = if true {
  \"some string\"
} else {
  12
}
switch! a {}

// A type in mers can consist of multiple single types: The type of a is 'string/int', because it could be either a string or an int.
// You can see this type mentioned in the error at the top of the file, which shows up because 'switch!' wants us to handle all possible types,
// yet we don't handle any ('{}').

// By combining tuples ('[a b c]') with the idea of multiple-types, you can create complex datastructures.
// You effectively have all the power of Rust enums (enums containing values) and structs combined:
// Rust's Option<T>:  t/[] or [t]/[] if t can be [] itself
// Rust's Result<T, E>: T/Err(E)

// The Err(E) is mers' version of an enum. An enum in mers is an identifier ('Err') wrapping a type ('E').
// They don't need to be declared anywhere. You can just return 'PossiblyWrongValue: 0.33' from your function and mers will handle the rest.
// To access the inner value, you can use the noenum() function:
// result = SomeValueInAnEnum: \"a string\"
// println(result) // error - result is not a string
// println(result.noenum()) // works because result is an enum containing a string

// the \\S+ regex matches anything but whitespaces
words_in_string = \"some string\".regex(\"\\\\S+\")
switch! words_in_string {}
// Types to cover: [string ...]/Err(string) - If the regex is invalid, regex() will return an error.

// To return to the menu, fix all compiler errors (comment out all switch! statements).

true
"));
    loop {
        match &tutor.let_user_make_change().run(vec![]).data().0 {
            VDataEnum::Tuple(v) if v.is_empty() => {
                tutor.set_status(format!(" - Returned an empty tuple."));
                tutor.update(None);
            }
            _ => break,
        }
    }
}
