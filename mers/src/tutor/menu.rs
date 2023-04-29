use crate::script::val_data::VDataEnum;

use super::Tutor;

pub const MAX_POS: usize = 7;

pub fn run(mut tutor: Tutor) {
    loop {
        tutor.current_pos = 0;
        tutor.update(Some(
            "
// Welcome to the mers tutor!
// This is the main menu. Change the number to navigate to a specific part.
fn go_to() 0
//   1  Comments
//   2  Functions
//   3  Values
//   4  Variables
//   5  Returns
//   6  Types
//   7  Error handling

go_to()
",
        ));
        loop {
            match tutor.let_user_make_change().run(vec![]).data {
                VDataEnum::Int(pos) if pos != 0 => {
                    tutor.current_pos = (pos.max(0) as usize).min(MAX_POS);
                    match tutor.current_pos {
                        0 => continue,
                        1 => super::base_comments::run(&mut tutor),
                        2 => super::base_functions::run(&mut tutor),
                        3 => super::base_values::run(&mut tutor),
                        4 => super::base_variables::run(&mut tutor),
                        5 => super::base_return::run(&mut tutor),
                        6 => super::base_types::run(&mut tutor),
                        7 => super::error_handling::run(&mut tutor),
                        _ => unreachable!(),
                    }
                }
                other => {
                    tutor.set_status(format!(
                        " - Returned {} instead of a nonzero integer",
                        other
                    ));
                }
            }
            break;
        }
    }
}
