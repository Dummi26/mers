use std::sync::Arc;

use crate::data::{
    self,
    function::Function,
    object::{Object, ObjectFieldsMap, ObjectT},
    string::StringT,
    tuple::{Tuple, TupleT},
    Data, Type,
};

use super::Config;

impl Config {
    pub fn with_fs(self) -> Self {
        self.add_var(
            "fs_read_text",
            Function::new_generic(
                |a, i| {
                    if a.is_included_in_single(&StringT) {
                        Ok(Type::newm(vec![
                            Arc::new(StringT),
                            Arc::new(ObjectT::new(vec![(
                                i.global.object_fields.get_or_add_field("fs_read_error"),
                                Type::new(data::string::StringT),
                            )])),
                        ]))
                    } else {
                        Err(format!(
                            "Called fs_read_text with argument type {}, but expected String",
                            a.with_info(i)
                        ))?
                    }
                },
                |a, i| {
                    let a = a.get();
                    let a = a
                        .as_any()
                        .downcast_ref::<data::string::String>()
                        .expect("got non-string argument to fs_read_text");
                    Ok(match std::fs::read_to_string(&a.0) {
                        Ok(contents) => Data::new(data::string::String(contents)),
                        Err(e) => Data::new(Object::new(vec![(
                            i.global.object_fields.get_or_add_field("fs_read_error"),
                            Data::new(data::string::String(e.to_string())),
                        )])),
                    })
                },
            ),
        )
        .add_var(
            "fs_write",
            Function::new_generic(
                |a, i| {
                    if a.is_included_in_single(&TupleT(vec![
                        Type::new(StringT),
                        Type::new(StringT),
                    ])) {
                        Ok(Type::newm(vec![
                            Arc::new(TupleT(vec![])),
                            Arc::new(ObjectT::new(vec![(
                                i.global.object_fields.get_or_add_field("fs_write_error"),
                                Type::new(data::string::StringT),
                            )])),
                        ]))
                    } else {
                        Err(format!(
                            "Called fs_write with argument type {}, but expected (String, String)",
                            a.with_info(i)
                        ))?
                    }
                },
                |a, i| {
                    let a = a.get();
                    let a = a
                        .as_any()
                        .downcast_ref::<Tuple>()
                        .expect("got non-tuple argument to fs_read_text");
                    let (a, b) = (a.0[0].get(), a.0[1].get());
                    let a = a
                        .as_any()
                        .downcast_ref::<data::string::String>()
                        .expect("file path was not a string in fs_write");
                    let b = b
                        .as_any()
                        .downcast_ref::<data::string::String>()
                        .expect("file content was not a string in fs_write");
                    Ok(match std::fs::write(&a.0, &b.0) {
                        Ok(()) => Data::empty_tuple(),
                        Err(e) => Data::new(Object::new(vec![(
                            i.global.object_fields.get_or_add_field("fs_write_error"),
                            Data::new(data::string::String(e.to_string())),
                        )])),
                    })
                },
            ),
        )
    }
}
