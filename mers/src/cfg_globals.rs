use std::sync::{Arc, RwLock};

use mers_lib::{
    data::{self, Data, Type},
    prelude_extend_config::*,
    program::configs::{self},
};

pub fn add_general(cfg: Config, args: Vec<String>) -> Config {
    cfg.add_var(
        "args",
        data::function::Function::new_static(
            vec![(
                Type::empty_tuple(),
                Type::new(configs::with_list::ListT(Type::new(data::string::StringT))),
            )],
            move |_, _| {
                Ok(Data::new(configs::with_list::List(
                    args.iter()
                        .map(|v| Arc::new(RwLock::new(Data::new(data::string::String(v.clone())))))
                        .collect(),
                )))
            },
        ),
    )
}
