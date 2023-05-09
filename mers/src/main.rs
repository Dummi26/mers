#![allow(unused)]
#![allow(dead_code)]

use std::{fs, time::Instant};

use notify::Watcher as FsWatcher;

mod interactive_mode;
mod libs;
#[cfg(feature = "nushell_plugin")]
mod nushell_plugin;
mod parsing;
mod script;
mod tutor;

fn main() {
    #[cfg(not(feature = "nushell_plugin"))]
    normal_main();
    #[cfg(feature = "nushell_plugin")]
    nushell_plugin::main();
}

fn normal_main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(debug_assertions)]
    let args = if args.len() == 0 {
        let mut args = args;
        args.push("../script.mers".to_owned());
        args
    } else {
        args
    };
    let mut file = match args.len() {
        0 => {
            println!("Please provide some arguments, such as the path to a file or \"-e <code>\".");
            std::process::exit(100);
        }
        _ => {
            if args[0].trim_start().starts_with("-") {
                let mut execute = false;
                let mut print_version = false;
                let mut verbose = 0;
                let mut interactive = 0;
                let mut interactive_use_new_terminal = false;
                let mut teachme = false;
                let mut prev_char = None;
                let mut advanced = false;
                for ch in args[0][1..].chars() {
                    if !advanced {
                        if ch == '+' {
                            advanced = true;
                            continue;
                        }
                        match ch {
                            'e' => execute = true,
                            'v' => verbose += 1,
                            'V' => print_version = true,
                            'i' => interactive += 1,
                            't' => teachme = true,
                            ch => {
                                eprintln!("Ignoring -{ch}. (unknown char)");
                                continue;
                            }
                        }
                        prev_char = Some(ch);
                    } else {
                        if let Some(prev_char) = prev_char {
                            match prev_char {
                                'i' => match ch {
                                    't' => interactive_use_new_terminal = true,
                                    _ => eprintln!("Ignoring i+{ch}. (unknown adv char)"),
                                },
                                _ => (),
                            }
                        } else {
                            eprintln!(
                                "Ignoring advanced args because there was no previous argument."
                            );
                        }
                        advanced = false;
                    }
                }
                if print_version {
                    println!(
                        "mers {}",
                        option_env!("CARGO_PKG_VERSION")
                            .unwrap_or("[[ version unknown: no CARGO_PKG_VERSION ]]")
                    );
                    return;
                }
                if teachme {
                    tutor::start(false);
                    return;
                }
                if verbose != 0 {
                    eprintln!("info: set verbosity level to {verbose}. this doesn't do anything yet. [TODO!]");
                }
                if interactive > 0 {
                    match interactive {
                        _ => {
                            // basic: open file and watch for fs changes
                            interactive_mode::fs_watcher::playground(interactive_use_new_terminal)
                        }
                    };
                    return;
                } else if execute {
                    parsing::file::File::new(
                        args.iter().skip(1).fold(String::new(), |mut s, v| {
                            if !s.is_empty() {
                                s.push(' ');
                            }
                            s.push_str(v);
                            s
                        }),
                        std::path::PathBuf::new(),
                    )
                } else {
                    println!("please provide either a file or -e and a script to run!");
                    std::process::exit(101);
                }
            } else {
                parsing::file::File::new(
                    std::fs::read_to_string(&args[0]).unwrap(),
                    args[0].as_str().into(),
                )
            }
        }
    };
    match parsing::parse::parse(&mut file) {
        Ok(script) => {
            println!(" - - - - -");
            let start = Instant::now();
            let out = script.run(std::env::args().skip(2).collect());
            let elapsed = start.elapsed();
            println!(" - - - - -");
            println!("Output ({}s)\n{out}", elapsed.as_secs_f64());
        }
        Err(e) => {
            println!("Couldn't compile:\n{}", e.with_file(&file));
            std::process::exit(99);
        }
    }
}
