use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    data::{
        self,
        int::{INT_MAX, INT_MIN},
        Data, MersDataWInfo, Type,
    },
    program::{self, run::CheckInfo},
};

use super::{
    gen::{
        function::{func, func_end},
        IntR, OneOrNone,
    },
    Config,
};

impl Config {
    /// `println: fn` prints to stdout and adds a newline to the end
    /// `print: fn` prints to stdout
    /// `eprintln: fn` prints to stderr and adds a newline to the end
    /// `eprint: fn` prints to stderr
    /// `debug: fn` debug-prints any value
    /// `read_line: fn` reads a line from stdin and returns it
    /// `exit: fn` exits the program with the given exit code. returns `<unreachable>`, just like `panic`
    pub fn with_stdio(self) -> Self {
        self
            //     .add_var(
            //     "read_line",
            //     data::function::Function {
            //         info: program::run::Info::neverused(),
            //         info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
            //         out: Ok(Arc::new(|a, _i| {
            //             if a.is_zero_tuple() {
            //                 Ok(Type::newm(vec![
            //                     Arc::new(data::tuple::TupleT(vec![Type::new(data::string::StringT)])),
            //                     Arc::new(data::tuple::TupleT(vec![])),
            //                 ]))
            //             } else {
            //                 Err(CheckError::new().msg(vec![
            //                     ("expected (), got ".to_owned(), None),
            //                     (a.to_string(), Some(EColor::FunctionArgument)),
            //                 ]))
            //             }
            //         })),
            //         run: Arc::new(|_a, _i| {
            //             Ok(if let Some(Ok(line)) = std::io::stdin().lines().next() {
            //                 Data::one_tuple(Data::new(data::string::String(line)))
            //             } else {
            //                 Data::empty_tuple()
            //             })
            //         }),
            //         inner_statements: None,
            //     },
            // )
            .add_var(
                "exit",
                func_end(|code: IntR<INT_MIN, INT_MAX>, _| {
                    std::process::exit(code.0.try_into().unwrap_or(255));
                }),
            )
            .add_var(
                "read_line",
                func(|_: (), _| {
                    Ok(if let Some(Ok(line)) = std::io::stdin().lines().next() {
                        OneOrNone(Some(line))
                    } else {
                        OneOrNone(None)
                    })
                }),
            )
            .add_var(
                "debug",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, _i| Ok(a.clone()))),
                    run: Arc::new(|a, i| {
                        let a2 = a.get();
                        eprintln!("{} :: {}", a2.as_type().with_info(i), a2.with_info(i));
                        drop(a2);
                        Ok(a)
                    }),
                    inner_statements: None,
                },
            )
            .add_var(
                "eprint",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|_a, _i| Ok(Type::empty_tuple()))),
                    run: Arc::new(|a, i| {
                        eprint!("{}", a.get().with_info(i));
                        _ = std::io::stderr().lock().flush();
                        Ok(Data::empty_tuple())
                    }),
                    inner_statements: None,
                },
            )
            .add_var(
                "eprintln",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|_a, _i| Ok(Type::empty_tuple()))),
                    run: Arc::new(|a, i| {
                        eprintln!("{}", a.get().with_info(i));
                        Ok(Data::empty_tuple())
                    }),
                    inner_statements: None,
                },
            )
            .add_var(
                "print",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|_a, _i| Ok(Type::empty_tuple()))),
                    run: Arc::new(|a, i| {
                        print!("{}", a.get().with_info(i));
                        _ = std::io::stdout().lock().flush();
                        Ok(Data::empty_tuple())
                    }),
                    inner_statements: None,
                },
            )
            .add_var(
                "println",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|_a, _i| Ok(Type::empty_tuple()))),
                    run: Arc::new(|a, i| {
                        println!("{}", a.get().with_info(i));
                        Ok(Data::empty_tuple())
                    }),
                    inner_statements: None,
                },
            )
    }
}
