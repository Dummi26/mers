use clap::{Parser, Subcommand, ValueEnum};
use mers_lib::prelude_compile::*;
use std::{fmt::Display, fs, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum, default_value_t = Configs::Std)]
    config: Configs,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Run { file: PathBuf },
    Exec { source: String },
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Configs {
    None,
    Std,
}
impl Display for Configs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Std => write!(f, "std"),
        }
    }
}

fn main() {
    let args = Args::parse();
    let config = match args.config {
        Configs::None => Config::new(),
        Configs::Std => Config::new().bundle_std(),
    };
    let (mut info1, mut info2) = config.infos();
    match args.command {
        Command::Run { file } => {
            let str = fs::read_to_string(file).unwrap();
            let mut src = Source::new(str);
            let parsed = parse(&mut src).unwrap();
            let run = parsed.compile(&mut info1, Default::default()).unwrap();
            run.run(&mut info2);
        }
        Command::Exec { source } => {
            let mut src = Source::new(source);
            let parsed = parse(&mut src).unwrap();
            let run = parsed.compile(&mut info1, Default::default()).unwrap();
            run.run(&mut info2);
        }
    }
}
