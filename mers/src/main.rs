#![allow(unused)]
#![allow(dead_code)]

use std::{fs, time::Instant};

use lang::global_info::ColorFormatMode;
use lang::global_info::GlobalScriptInfo;
use lang::global_info::LogKind;
use notify::Watcher as FsWatcher;

use crate::lang::fmtgs::FormatGs;

mod interactive_mode;
mod lang;
mod libs;
#[cfg(feature = "nushell_plugin")]
mod nushell_plugin;
mod parsing;
mod tutor;

fn main() {
    #[cfg(not(feature = "nushell_plugin"))]
    normal_main();
    #[cfg(feature = "nushell_plugin")]
    nushell_plugin::main();
}

fn normal_main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut info = GlobalScriptInfo::default();
    let mut run = true;
    let mut args_to_skip = 2;
    let mut file = match args.len() {
        0 => {
            println!("Please provide some arguments, such as the path to a file or \"-e <code>\".");
            std::process::exit(100);
        }
        _ => {
            if args[0].trim_start().starts_with("-") {
                let mut execute = false;
                let mut print_version = false;
                let mut verbose = false;
                let mut verbose_args = String::new();
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
                            'v' => verbose = true,
                            'f' => {
                                run = false;
                                info.log.after_parse.stderr = true;
                            }
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
                        advanced = false;
                        if let Some(prev_char) = prev_char {
                            match prev_char {
                                'i' => match ch {
                                    't' => interactive_use_new_terminal = true,
                                    _ => eprintln!("Ignoring i+{ch}. (unknown adv char)"),
                                },
                                'v' => {
                                    if ch != '+' {
                                        advanced = true;
                                        verbose_args.push(ch);
                                    }
                                }
                                'f' => match ch {
                                    'c' => info.formatter.mode = ColorFormatMode::Colorize,
                                    'C' => info.formatter.mode = ColorFormatMode::Plain,
                                    _ => eprintln!("Ignoring f+{ch}. (unknown adv char)"),
                                },
                                _ => (),
                            }
                        } else {
                            eprintln!(
                                "Ignoring advanced args because there was no previous argument."
                            );
                        }
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
                if verbose {
                    if verbose_args.is_empty() {
                        fn f() -> LogKind {
                            LogKind {
                                stderr: true,
                                log: true,
                            }
                        }
                        info.log.vdata_clone = f();
                        info.log.vtype_fits_in = f();
                        info.log.vsingletype_fits_in = f();
                    } else {
                        fn kind(val: Option<&str>) -> LogKind {
                            match val {
                                Some("stderr") => LogKind {
                                    stderr: true,
                                    ..Default::default()
                                },
                                Some("log") => LogKind {
                                    log: true,
                                    ..Default::default()
                                },
                                Some("log+stderr" | "stderr+log") => LogKind {
                                    stderr: true,
                                    log: true,
                                    ..Default::default()
                                },
                                _ => LogKind {
                                    stderr: true,
                                    ..Default::default()
                                },
                            }
                        }
                        for verbose_arg in verbose_args.split(',') {
                            let (arg, val) = match verbose_arg.split_once('=') {
                                Some((left, right)) => (left, Some(right)),
                                None => (verbose_arg, None),
                            };
                            match arg {
                                "vdata_clone" => info.log.vdata_clone = kind(val),
                                "vtype_fits_in" => info.log.vtype_fits_in = kind(val),
                                "vsingletype_fits_in" => info.log.vsingletype_fits_in = kind(val),
                                _ => eprintln!("Warn: -v+ unknown arg '{arg}'."),
                            }
                        }
                    }
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
                    args_to_skip += 1;
                    if let Some(file) = args.get(1) {
                        parsing::file::File::new(
                            std::fs::read_to_string(file).unwrap(),
                            file.into(),
                        )
                    } else {
                        println!("please provide either a file or -e and a script to run!");
                        std::process::exit(101);
                    }
                }
            } else {
                parsing::file::File::new(
                    std::fs::read_to_string(&args[0]).unwrap(),
                    args[0].as_str().into(),
                )
            }
        }
    };
    match parsing::parse::parse_custom_info(&mut file, info) {
        Ok(script) => {
            if run {
                script.run(std::env::args().skip(args_to_skip).collect());
            }
        }
        Err(e) => {
            println!("Couldn't compile:\n{}", e.with_file(&file));
            std::process::exit(99);
        }
    }
}
