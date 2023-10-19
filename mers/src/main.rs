use clap::{Parser, Subcommand, ValueEnum};
use mers_lib::prelude_compile::*;
use std::{fmt::Display, fs, path::PathBuf};

mod cfg_globals;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
    /// controls availability of features when compiling/running
    #[arg(long, value_enum, default_value_t = Configs::Std)]
    config: Configs,
    /// perform checks to avoid runtime crashes
    #[arg(long, default_value_t = Check::Yes)]
    check: Check,
}
#[derive(Subcommand)]
enum Command {
    /// runs the file
    Run { file: PathBuf },
    /// runs cli argument
    Exec { source: String },
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Check {
    No,
    Yes,
    Only,
}
impl Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::No => "no",
                Self::Yes => "yes",
                Self::Only => "only",
            }
        )
    }
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Configs {
    None,
    Base,
    Std,
}
impl Display for Configs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Base => write!(f, "base"),
            Self::Std => write!(f, "std"),
        }
    }
}

fn main() {
    let args = Args::parse();
    let config = cfg_globals::add_general(match args.config {
        Configs::None => Config::new(),
        Configs::Base => Config::new().bundle_base(),
        Configs::Std => Config::new().bundle_std(),
    });
    let (mut info_parsed, mut info_run, mut info_check) = config.infos();
    let mut source = match args.command {
        Command::Run { file } => {
            let str = fs::read_to_string(file).unwrap();
            Source::new(str)
        }
        Command::Exec { source } => Source::new(source),
    };
    let parsed = parse(&mut source).unwrap();
    #[cfg(debug_assertions)]
    dbg!(&parsed);
    let run = parsed
        .compile(&mut info_parsed, Default::default())
        .unwrap();
    #[cfg(debug_assertions)]
    dbg!(&run);
    match args.check {
        Check::No => {
            run.run(&mut info_run);
        }
        Check::Yes | Check::Only => {
            let return_type = match run.check(&mut info_check, None) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("check failed: {e}");
                    std::process::exit(36);
                }
            };
            #[cfg(debug_assertions)]
            dbg!(&return_type);
            if args.check == Check::Yes {
                run.run(&mut info_run);
            } else {
                eprintln!("return type is {}", return_type)
            }
        }
    }
}
