use crate::script::val_data::VDataEnum;

use super::Tutor;

pub const MAX_POS: usize = 3;

pub fn run(mut tutor: Tutor) {
    loop {
        tutor.current_pos = 0;
        tutor.update(Some(
            "
// Welcome to the mers tutor!
// This is the main menu. Change the number to navigate to a specific part.
0
//   1  Comments
//   2  Values
//   3  Returns
",
        ));
        loop {
            match tutor.let_user_make_change().run(vec![]).data {
                VDataEnum::Int(pos) => {
                    tutor.current_pos = (pos.max(0) as usize).min(MAX_POS);
                    match tutor.current_pos {
                        0 => continue,
                        1 => super::base_comments::run(&mut tutor),
                        2 => super::base_values::run(&mut tutor),
                        3 => super::base_return::run(&mut tutor),
                        _ => unreachable!(),
                    }
                }
                other => {
                    tutor.set_status(format!(" - Returned {} instead of an integer", other));
                }
            }
            break;
        }
    }
}
