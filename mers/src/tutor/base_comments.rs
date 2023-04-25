use crate::script::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some(
        "
// Comments in mers start at // and end at the end of the line.
// They also work within strings, which can be unexpected in some cases (like \"http://www...\").
/* also works to start a comment.
   This comment can even span multiple lines! */
// To return to the menu, uncomment the next line:
// true
",
    ));
    loop {
        match tutor.let_user_make_change().run(vec![]).data {
            VDataEnum::Bool(true) => break,
            other => {
                tutor.set_status(format!(" - Returned {} instead of true.", other));
                tutor.update(None);
            }
        }
    }
}
