use clap::{Parser, Subcommand, ValueEnum};
use mers_lib::prelude_compile::*;
use std::{path::PathBuf, process::exit, sync::Arc};

mod cfg_globals;
mod pretty_print;

#[derive(Parser)]
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
        source: From,
    },
    /// Run code, but skip type-checks. Will panic at runtime if code is not valid.
    RunUnchecked {
        #[command(subcommand)]
        source: From,
    },
    /// Add syntax highlighting to the code
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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Configs {
    None,
    Base,
    Pure,
    Std,
}

fn main() {
    let args = Args::parse();
    let config = cfg_globals::add_general(match args.config {
        Configs::None => Config::new(),
        Configs::Base => Config::new().bundle_base(),
        Configs::Pure => Config::new().bundle_pure(),
        Configs::Std => Config::new().bundle_std(),
    });
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
                    eprintln!("{e}");
                    exit(20);
                }
                Ok(parsed) => {
                    let (i1, _, i3) = config.infos();
                    match compile(&*parsed, i1) {
                        Err(e) => {
                            eprintln!("{e}");
                            exit(24);
                        }
                        Ok(compiled) => match check(&*compiled, i3) {
                            Err(e) => {
                                eprintln!("{e}");
                                exit(28);
                            }
                            Ok(output_type) => eprintln!("{output_type}"),
                        },
                    }
                }
            }
        }
        Command::Run { source } => {
            let mut src = get_source(source);
            let srca = Arc::new(src.clone());
            match parse(&mut src, &srca) {
                Err(e) => {
                    eprintln!("{e}");
                    exit(255);
                }
                Ok(parsed) => {
                    let (i1, mut i2, i3) = config.infos();
                    match compile(&*parsed, i1) {
                        Err(e) => {
                            eprintln!("{e}");
                            exit(255);
                        }
                        Ok(compiled) => match check(&*compiled, i3) {
                            Err(e) => {
                                eprintln!("{e}");
                                exit(255);
                            }
                            Ok(_) => {
                                if let Err(e) = compiled.run(&mut i2) {
                                    eprintln!("Error while running:\n{e}");
                                    std::process::exit(1);
                                }
                            }
                        },
                    }
                }
            }
        }
        Command::RunUnchecked { source } => {
            let mut src = get_source(source);
            let srca = Arc::new(src.clone());
            match parse(&mut src, &srca) {
                Err(e) => {
                    eprintln!("{e}");
                    exit(255);
                }
                Ok(parsed) => {
                    let (i1, mut i2, _) = config.infos();
                    match compile(&*parsed, i1) {
                        Err(e) => {
                            eprintln!("{e}");
                            exit(255);
                        }
                        Ok(compiled) => {
                            if let Err(e) = compiled.run(&mut i2) {
                                eprintln!("Error while running:\n{e}");
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        }
        Command::PrettyPrint { source } => {
            pretty_print::pretty_print(get_source(source));
        }
    }
}
