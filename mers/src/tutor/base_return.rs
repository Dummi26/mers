use crate::script::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some(
        "
// Mers doesn't have a return statement.
// Instead, the value of the last statement is implicitly returned.

// This applies to blocks:
b = {
    a = 10
    a = a.add(15)
    a
}
// b = 25

// To functions:
fn compute_sum(a int b int) {
    a.add(b)
}
// returns a+b

// and to the program itself!
// to return to the menu, make the program return 15.
",
    ));
    loop {
        match tutor.let_user_make_change().run(vec![]).data {
            VDataEnum::Int(15) => break,
            other => {
                tutor.set_status(format!(" - Returned {} instead of 15.", other));
                tutor.update(None);
            }
        }
    }
}
