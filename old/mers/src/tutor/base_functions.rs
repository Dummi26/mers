use crate::lang::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some(
        "
// Functions represent certain actions.
// They are given some inputs (arguments) and output (return) something.
// Mers comes with a range of builtin functions defined in src/script/builtins.rs.

// As an example, let's look at the add() function:
// It takes two arguments as its input and adds them together, then returns the sum:
add(5 10) // 15
// Similar to this, sub() subtracts two numbers:
sub(15 5) // 10

// For some functions, there is no value they could return:
sleep(0.01) // wait 0.01 seconds, then continue.
// These will return an empty tuple [] in mers.

// However, you aren't limited to the builtin functions.
// You can easily define your own functions to do more complex tasks:
fn say_hello_world() {
    println(\"Hello, world!\")
}

// Since the Subject.Verb(Object) syntax is more natural to many people, a.function(b c d) is an alternative way of writing function(a b c d):
my_var = 15
format(\"my variable had the value {0}!\" my_var) // normal
\"my variable had the value {0}!\".format(my_var) // alternative (does the same thing)

// to return to the menu, add two arguments to the mul() function to make it return 32*5
mul()
",
    ));
    loop {
        match tutor.let_user_make_change().run(vec![]).inner_cloned() {
            VDataEnum::Int(160) => break,
            other => {
                tutor.set_status(format!(" - Returned {other} instead of 160"));
                tutor.update(None);
            }
        }
    }
}
