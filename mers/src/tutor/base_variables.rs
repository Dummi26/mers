use crate::script::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some(
        "
// A variable can be used to store values.
// Create one by assigning a value to it:
my_first_variable = 15
// Then use it instead of literal values:
five_less = sub(my_first_variable 5) // 10

// to return to the menu, create a variable my_name and assign your name to it.


/* return the name so the tutor can check it - ignore this */ my_name
",
    ));
    loop {
        match tutor.let_user_make_change().run(vec![]).data {
            VDataEnum::String(name) if !name.is_empty() => {
                tutor.i_name = Some(name);
                break;
            }
            VDataEnum::String(_) => {
                tutor.set_status(format!(" - Almost there, you made an empty string. Put your name between the quotes to continue!"));
                tutor.update(None);
            }
            other => {
                tutor.set_status(format!(" - Returned {other} instead of a string. String literals start and end with double quotes (\")."));
                tutor.update(None);
            }
        }
    }
}
