use std::sync::Arc;

use crate::{
    data::{self, Data},
    program,
};

use super::Config;

impl Config {
    /// `println: fn` prints to stdout and adds a newline to the end
    /// `print: fn` prints to stdout
    /// `eprintln: fn` prints to stderr and adds a newline to the end
    /// `eprint: fn` prints to stderr
    /// `debug: fn` debug-prints any value
    pub fn with_prints(self) -> Self {
        self.add_var(
            "debug".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    eprintln!("{:#?}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "eprint".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    eprint!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "eprintln".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    eprintln!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "print".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    print!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "println".to_string(),
            Data::new(data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_a| todo!()),
                run: Arc::new(|a, _i| {
                    println!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
    }
}
