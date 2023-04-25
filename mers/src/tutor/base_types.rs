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

// Traditional statically-typed languages achieve this same type-safety:
  // C / C++ / Java / C#
    //  int a = 10;
    //  int b = 5;
    //  int c = a - b;
  // In C#, we can just use 'var' to automatically infer the types
    //  var a = 10;
    //  var b = 5;
    //  var c = a - b; // all of these are ints, and C# knows this
  // Not specifying a type for variables is the default in Rust...
    //  let a = 10;
    //  let b = 5;
    //  let c = a - b;
  // ... and Go
    // a := 10
    // b := 5
    // c := a - b
// Dynamically-typed languages don't need to know the type of their variables at all:
  // JavaScript
    //  let a = 10
    //  let b = 5
    //  let c = a - b
  // Also JavaScript (c becomes NaN in this example)
    //  let a = \"some text\"
    //  let b = 5
    //  let c = a - b
// However, there are some things dynamic typing can let us do that static typing can't:
  // JavaScript
    //  let x
    //  if (condition()) {
    //    x = 10
    //  } else {
    //    x = \"some string\"
    //  }
    //  console.log(x) // we can't know if x is an int or a string, but it will work either way.
// We *could* implement this in Rust:
  //  enum StringOrInt {
  //    S(String),
  //    I(i32),
  //  }
  //  impl std::fmt::Display for StringOrInt {
  //    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
  //      match self {
  //        Self::S(s) => write!(f, \"{}\", s),
  //        Self::I(i) => write!(f, \"{}\", i),
  //      }
  //    }
  //  }
  //  fn main() {
  //      let x;
  //      if condition() {
  //        x = StringOrInt::I(10)
  //      } else {
  //        x = StringOrInt::S(format!(\"some string\"));
  //      }
  //      println!(\"{}\", x);
  //  }
// While this works, it's a lot of effort. But don't worry, we can do better!
  // Mers (it doesn't let you declare variables without initializing them, but that doesn't really matter for the example)
    //  x = if condition() {
    //    10
    //  } else {
    //    \"some string\"
    //  }
    //  println(\"x = {0}\".format(x))
// Okay, but how can we keep the type-safety of statically typed languages if code like this is valid in mers?
// To figure this out, let's just ask mers for the type of x using 'switch! var {}'.
// (you can always reload and check the top of the file to see mers' full error)

a = \"this is clearly a string\"
switch! a {} // string

b = 10
switch! b {} // int

// Now comment out the two switch statements, reload the file and look at the error produced by 'switch! x {}'.

x = if true {
  10
} else {
  \"a string\"
}
switch! x {}

// Mers doesn't know if x is an int or a string, but it knows that it has to be either of those and can't be anything else.
// And this doesn't even rely on any fancy technology, it's all just based on simple and intuitive rules:
// x is the result of an if-else statement, meaning x's type is just the return types from the two branches combined: int and string -> int/string.
// if we remove the else, x's type would be int/[]: either an int or nothing.
// (don't forget to comment out the third switch statement, too. you can't return to the menu otherwise)

// By combining multiple-types with tuples, we can express complicated data structures without needing structs or enums:
// [int/float int/float]/int/float // one or two numbers
// [int/float ...] // a list of numbers

// Mers does have enums, but they are different from what other languages have.
// Enums are just identifiers that can be used to specify what a certain type is supposed to be used for.
// Consider a function read_file() that wants to return a the file's contents as a string.
// If the file doesn't exist or can't be read, the function should return some sort of error.
// Both of these can be the type string: \"this is my file\" and \"Couldn't read file: Permission denied.\".
// To avoid this ambiguity, we can wrap the error in an enum:
// \"this is my file\" and Err: \"Couldn't read file: Permission denied.\".
// Instead of the function just returning string, it now returns string/Err(string).
// This shows programmers that your function can fail and at the same time tells mers
// that the function returns two different types that both need to be handeled:
fn read_file() {
    if false {
      // successfully read the file
      \"this is my file\"
    } else {
      Err: \"Couldn't read file: I didn't even try.\"
    }
}
file = read_file()
// without switching, I can't get to the file's content:
println(file) // this causes an error!
// using switch! instead of switch forces me to handle all types, including the error path.
switch! file {
  string {
    println(\"File content: {0}\".format(file))
  }
  Err(string) {
    println(\"Error! {0}\".format(file.noenum()))
  }
}

// To return to the menu, fix all the errors in this file.

true
"));
    loop {
        match tutor.let_user_make_change().run(vec![]).data {
            VDataEnum::Tuple(v) if v.is_empty() => {
                tutor.set_status(format!(" - Returned an empty tuple."));
                tutor.update(None);
            }
            _ => break,
        }
    }
}
