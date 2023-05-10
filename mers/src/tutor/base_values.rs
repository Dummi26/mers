use crate::script::val_data::VDataEnum;

use super::Tutor;

pub fn run(tutor: &mut Tutor) {
    tutor.update(Some("
// Mers has the following values:
//   bool: Either true or false
//   int: An integer. Written 0, 1, 2, 3, -5, and so on
//   float: A floating point number. Written 0.5, -12.378, and so on
//   string: A piece of text. Surround something with \" and it will be a string: \"Hello, world!\"
//   tuples: Multiple values in one place. A fixed-length collection. Surround types or statements with [] to create a tuple: [12 \"a tuple of ints and a string\" -5 -12]
//           The empty tuple [] is often used to indicate nothing, while a 1-long tuple [v] indicates the opposite - something.
//   list: Similar to tuples, but the closing ] is prefixed with 3 dots: [ ...]
//         Unlike tuples, all elements in a list have the same type. Lists are resizable and can grow dynamically, while tuples cannot change their size after being created.
//   function: A piece of code in data-form.
//             value: anonymous_sum_function = (a int/float b int/float) a.add(b)
//              type: fn((int int int)(int float float)(float int float)(float float float))
//                    the reason why the type syntax is so expressive is because the function doesn't return the same type for any inputs - add will return an int if it added two ints, but will return a float when at least one argument was a float.
//                    add will NOT return int/float, because if you know the exact input types, you also know the output type: either int and not float or float and not int.
//   thread: Represents a different thread. The thread's return value can be retrieved by using .await(). Thread values are returned by the builtin thread() function.
//   reference: A mutable reference to some data. Used by things like push() and remove() to avoid having to clone the entire list just to make a small change.
//   enums: An enum can wrap any type. Enums are identified by their names and can be created using EnumName: inner_value. The type is written EnumName(InnerType).
// return any enum to return to the menu.
"));
    loop {
        match &tutor.let_user_make_change().run(vec![]).data().0 {
            VDataEnum::EnumVariant(..) => break,
            other => {
                tutor.set_status(format!(" - Returned {other} instead of an enum."));
                tutor.update(None);
            }
        }
    }
}
