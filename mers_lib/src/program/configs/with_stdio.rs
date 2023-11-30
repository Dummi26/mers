use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use colored::Colorize;

use crate::{
    data::{self, Data, Type},
    errors::error_colors,
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// `println: fn` prints to stdout and adds a newline to the end
    /// `print: fn` prints to stdout
    /// `eprintln: fn` prints to stderr and adds a newline to the end
    /// `eprint: fn` prints to stderr
    /// `debug: fn` debug-prints any value
    /// `read_line: fn` reads a line from stdin and returns it
    pub fn with_stdio(self) -> Self {
        self.add_var(
            "read_line".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_zero_tuple() {
                        Ok(Type::new(data::string::StringT))
                    } else {
                        Err(format!(
                            "expected (), got {}",
                            a.to_string().color(error_colors::FunctionArgument)
                        )
                        .into())
                    }
                }),
                run: Arc::new(|_a, _i| {
                    let mut line = String::new();
                    _ = std::io::stdin().read_line(&mut line);
                    Data::new(data::string::String(line))
                }),
            }),
        )
        .add_var(
            "debug".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|_a, _i| Ok(Type::empty_tuple())),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    eprintln!("{} :: {}", a.as_type(), a);
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "eprint".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|_a, _i| Ok(Type::empty_tuple())),
                run: Arc::new(|a, _i| {
                    eprint!("{}", a.get());
                    _ = std::io::stderr().lock().flush();
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "eprintln".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|_a, _i| Ok(Type::empty_tuple())),
                run: Arc::new(|a, _i| {
                    eprintln!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "print".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|_a, _i| Ok(Type::empty_tuple())),
                run: Arc::new(|a, _i| {
                    print!("{}", a.get());
                    _ = std::io::stdout().lock().flush();
                    Data::empty_tuple()
                }),
            }),
        )
        .add_var(
            "println".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|_a, _i| Ok(Type::empty_tuple())),
                run: Arc::new(|a, _i| {
                    println!("{}", a.get());
                    Data::empty_tuple()
                }),
            }),
        )
    }
}
