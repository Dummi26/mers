use lang::global_info::ColorFormatMode;
use lang::global_info::GlobalScriptInfo;
use lang::global_info::LogKind;
use lang::val_data::VDataEnum;
use lang::val_type::VSingleType;

use crate::lang::fmtgs::FormatGs;

mod interactive_mode;
mod lang;
mod libs;
#[cfg(feature = "nushell_plugin")]
mod nushell_plugin;
mod parsing;
mod pathutil;
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
            println!("no arguments, use -h for help");
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
                            'h' => {
                                eprintln!("~~~~ mers help ~~~~");
                                eprintln!();
                                eprintln!("  ~~ cli ~~");
                                eprintln!("Mers has the following cli options:");
                                eprintln!("-h shows this Help message");
                                eprintln!("-e - mers will treat the run argument as code to be Executed rather than a file path");
                                eprintln!("    mers -e 'println(\"Hello, World!\")'");
                                eprintln!(
                                    "-c - mers will Check the code for errors, but won't run it"
                                );
                                eprintln!("-f - mers will Format the code and print it. useful if you suspect the parser might be misinterpreting your code");
                                eprintln!(
                                    "+c - use Colors in the output to better visualize things"
                                );
                                eprintln!("+C - don't use colors (opposite of +c, redundant since this is the default)");
                                eprintln!("-v - mers will be more Verbose");
                                eprintln!("+???+ - customize what mers is verbose about and how - bad syntax, barely useful, don't use it until it gets improved (TODO!)");
                                eprintln!("-i - launches an Interactive session to play around with (opens your editor and runs code on each file save)");
                                eprintln!("+t - spawns a new terminal for the editor (if you use a terminal editors, add +t)");
                                eprintln!("    mers -i+t");
                                eprintln!("-t - launches the Tutor, which will attempt to Teach you the basics of the language");
                                eprintln!();
                                eprintln!("  ~~ getting started ~~");
                                eprintln!("mers doesn't need a specific structure for directories, just create a UTF-8 text file, write code, and run it:");
                                eprintln!("    echo 'println(\"Hello, World!\")' > hello.mers");
                                eprintln!("    mers hello.mers");
                                return;
                            }
                            'e' => execute = true,
                            'v' => verbose = true,
                            'c' => run = false,
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
                                .unwrap()
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
                        println!("nothing to do - missing arguments?");
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
    info.main_fn_args = vec![(
        "args".to_string(),
        VSingleType::List(VSingleType::String.into()).to(),
    )];
    match parsing::parse::parse_custom_info(&mut file, info) {
        Ok(script) => {
            if run {
                script.run(vec![VDataEnum::List(
                    VSingleType::String.to(),
                    std::env::args()
                        .skip(args_to_skip)
                        .map(|v| VDataEnum::String(v).to())
                        .collect(),
                )
                .to()]);
            }
        }
        Err(e) => {
            println!("Couldn't compile:\n{}", e.with_file(&file));
            std::process::exit(99);
        }
    }
}
