use clap::{Parser, Subcommand, ValueEnum};
use mers_lib::prelude_compile::*;
use std::{path::PathBuf, process::exit, sync::Arc};

mod cfg_globals;

#[derive(Parser)]
#[command(version,
    about = Some("mers - a type-checked, dynamically typed programming language focusing on safety and simplicity.\n        run with --help for more info :)"),
    long_about = Some("mers - a type-checked, dynamically typed programming language focusing on safety and simplicity.
Safety in mers means that a valid, type-checked mers program will not crash at runtime, and that mers tries to make writing \"bad\" code difficult.
Simplicity means that mers is easy to learn, it has little syntax and few edge-cases, but it does not mean that it is easy to write."))]
struct Args {
    #[command(subcommand)]
    command: Command,
    /// controls availability of features when compiling/running
    #[arg(long, value_enum, default_value_t = Configs::Std)]
    config: Configs,
    /// in error messages, hide comments and only show actual code
    #[arg(long)]
    hide_comments: bool,
}
#[derive(Subcommand)]
enum Command {
    /// Check if code is valid. If yes, print output type.
    ///
    /// Exit status is 20 for parse errors, 24 for compile errors and 28 for check errors (type errors).
    Check {
        #[command(subcommand)]
        source: From,
    },
    /// Check and then run code. Exit status is 255 if checks fail.
    Run {
        #[command(subcommand)]
        source: FromArgs,
    },
    /// Run code, but skip type-checks. Will panic at runtime if code is not valid.
    RunUnchecked {
        #[command(subcommand)]
        source: FromArgs,
    },
    /// Not available, because the colored-output default feature was disabled when building mers!
    #[cfg(not(feature = "colored-output"))]
    PrettyPrint {
        #[command(subcommand)]
        source: From,
    },
    /// Add syntax highlighting to the code
    #[cfg(feature = "colored-output")]
    PrettyPrint {
        #[command(subcommand)]
        source: From,
    },
}
#[derive(Subcommand, Clone)]
enum From {
    /// runs the file
    File { file: PathBuf },
    /// runs cli argument
    Arg { source: String },
}
#[derive(Subcommand, Clone)]
enum FromArgs {
    /// runs the file
    File {
        file: PathBuf,
        #[arg(num_args=0..)]
        args: Vec<String>,
    },
    /// runs cli argument
    Arg {
        source: String,
        #[arg(num_args=0..)]
        args: Vec<String>,
    },
}
impl FromArgs {
    pub fn to(self) -> From {
        match self {
            Self::File { file, args: _ } => From::File { file },
            Self::Arg { source, args: _ } => From::Arg { source },
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Configs {
    None,
    Base,
    Pure,
    Std,
}

fn main() {
    let mut args = Args::parse();
    let config = cfg_globals::add_general(
        match args.config {
            Configs::None => Config::new(),
            Configs::Base => Config::new().bundle_base(),
            Configs::Pure => Config::new().bundle_pure(),
            Configs::Std => Config::new().bundle_std(),
        },
        match &mut args.command {
            Command::Run { source } | Command::RunUnchecked { source } => match source {
                FromArgs::File { file: _, args } | FromArgs::Arg { source: _, args } => {
                    std::mem::replace(args, vec![])
                }
            },
            _ => vec![],
        },
    );
    fn get_source(source: From) -> Source {
        match source {
            From::File { file } => match Source::new_from_file(PathBuf::from(&file)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Can't read file {file:?}: {e}");
                    exit(10);
                }
            },
            From::Arg { source } => Source::new_from_string(source),
        }
    }
    match args.command {
        Command::Check { source } => {
            let mut src = get_source(source);
            let srca = Arc::new(src.clone());
            match parse(&mut src, &srca) {
                Err(e) => {
                    eprintln!("{e:?}");
                    exit(20);
                }
                Ok(parsed) => {
                    let (i1, _, mut i3) = config.infos();
                    i3.global.show_warnings_to_stderr();
                    match compile(&*parsed, i1) {
                        Err(e) => {
                            eprintln!("{e:?}");
                            exit(24);
                        }
                        Ok(compiled) => match check_mut(&*compiled, &mut i3) {
                            Err(e) => {
                                eprintln!("{e:?}");
                                exit(28);
                            }
                            Ok(output_type) => eprintln!("{}", output_type.with_info(&i3)),
                        },
                    }
                }
            }
        }
        Command::Run { source } => {
            let mut src = get_source(source.to());
            let srca = Arc::new(src.clone());
            match parse(&mut src, &srca) {
                Err(e) => {
                    eprintln!("{e:?}");
                    exit(255);
                }
                Ok(parsed) => {
                    let (i1, mut i2, i3) = config.infos();
                    match compile(&*parsed, i1) {
                        Err(e) => {
                            eprintln!("{e:?}");
                            exit(255);
                        }
                        Ok(compiled) => match check(&*compiled, i3) {
                            Err(e) => {
                                eprintln!("{e:?}");
                                exit(255);
                            }
                            Ok(_) => {
                                if let Err(e) = compiled.run(&mut i2) {
                                    eprintln!("Error while running:\n{e:?}");
                                    std::process::exit(1);
                                }
                            }
                        },
                    }
                }
            }
        }
        Command::RunUnchecked { source } => {
            let mut src = get_source(source.to());
            let srca = Arc::new(src.clone());
            match parse(&mut src, &srca) {
                Err(e) => {
                    eprintln!("{e:?}");
                    exit(255);
                }
                Ok(parsed) => {
                    let (i1, mut i2, _) = config.infos();
                    match compile(&*parsed, i1) {
                        Err(e) => {
                            eprintln!("{e:?}");
                            exit(255);
                        }
                        Ok(compiled) => {
                            if let Err(e) = compiled.run(&mut i2) {
                                eprintln!("Error while running:\n{e:?}");
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        }
        #[cfg(feature = "colored-output")]
        Command::PrettyPrint { source } => {
            mers_lib::pretty_print::pretty_print(get_source(source));
        }
        #[cfg(not(feature = "colored-output"))]
        Command::PrettyPrint { source: _ } => {
            eprintln!("feature colored-output must be enabled when compiling mers if you want to use pretty-print!");
            std::process::exit(180);
        }
    }
}
