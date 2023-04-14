use std::{fs, sync::Arc, time::Instant};

use notify::Watcher as FsWatcher;

pub mod libs;
pub mod parse;
pub mod script;

// necessary because the lib target in Cargo.toml also points here. TODO: update Cargo.toml to have a lib target that is separate from the bin one (=> doesn't point to main)
#[allow(unused)]
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let path = std::env::args().nth(1).unwrap();
    let script = parse::parse::parse(&mut match args.len() {
        0 => {
            println!("Please provide some arguments, such as the path to a file or \"-e <code>\".");
            std::process::exit(100);
        }
        _ => {
            if args[0].trim_start().starts_with("-") {
                let mut execute = false;
                let mut verbose = 0;
                let mut interactive = 0;
                let mut interactive_use_new_terminal = false;
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
                            'i' => interactive += 1,
                            ch => {
                                eprintln!("Ignoring -{ch}. (unknown char)");
                                continue;
                            }
                        }
                        prev_char = Some(ch);
                    } else {
                        if let Some(prev_char) = prev_char {
                            match prev_char {
                                'i' => {
                                    match ch {
                                        't' => interactive_use_new_terminal = true,
                                        _ => eprintln!("Ignoring i+{ch}. (unknown adv char)"),
                                    }
                                }
                                _ => (),
                            }
                        } else {
                            eprintln!("Ignoring advanced args because there was no previous argument.");
                        }
                        advanced = false;
                    }
                }
                if verbose != 0 {
                    eprintln!("info: set verbosity level to {verbose}. this doesn't do anything yet. [TODO!]");
                }
                if interactive >= 0 {
                    let (contents, path) = match interactive {
                        1 => {
                            // basic: open file and watch for fs changes
                            let temp_file_edit = edit::Builder::new().suffix(".mers").tempfile().unwrap();
                            let temp_file = temp_file_edit.path();
                            eprintln!("Using temporary file at {temp_file:?}. Save the file to update the output here.");
                            if let Ok(_) = std::fs::write(&temp_file, []) {
                                if let Ok(mut watcher) = {
                                    let temp_file = temp_file.to_path_buf();
                                    // the file watcher
                                    notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                                    if let Ok(event) = event {
                                        match &event.kind {
                                            notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                                                if let Ok(file_contents) = fs::read_to_string(&temp_file) {
                                                    let mut file = parse::file::File::new(file_contents, temp_file.clone());
                                                        static_assertions::const_assert_eq!(parse::parse::PARSE_VERSION, 0);
                                                        let mut ginfo = script::block::to_runnable::GInfo::default();
                                                        let libs = parse::parse::parse_step_lib_paths(&mut file);
                                                        match parse::parse::parse_step_interpret(&mut file) {
                                                            Ok(func) => {
                                                        let libs = parse::parse::parse_step_libs_load(libs, &mut ginfo);
                                                                ginfo.libs = Arc::new(libs);
                                                        match parse::parse::parse_step_compile(func, &mut ginfo) {
                                                                    Ok(func) => {
                                                                        println!();
                                                                        println!(" - - - - -");
                                                                        let output = func.run(vec![]);
                                                                        println!(" - - - - -");
                                                                        println!("{}", output);
                                                                    }
                                                                    Err(e) => eprintln!("Couldn't compile:\n{e:?}"),
                                                                }
                                                            }
                                                            Err(e) =>eprintln!("Couldn't interpret:\n{e:?}"),
                                                        }
                                                } else {
                                                        println!("can't read file at {:?}!", temp_file);
                                                        std::process::exit(105);
                                                    }
                                            }
                                            _ => (),
                                        }
                                    }
                                })} {
                                    if let Ok(_) = watcher.watch(&temp_file, notify::RecursiveMode::NonRecursive) {
                                        if interactive_use_new_terminal {
                                            if let Ok(term) = std::env::var("TERM") {
                                                let editor = edit::get_editor().unwrap();
                                                eprintln!("launching \"{term} -e {editor:?} {temp_file:?}...");
                                                std::process::Command::new(term)
                                                    .arg("-e")
                                                    .arg(&editor)
                                                    .arg(temp_file)
                                                    .spawn()
                                                    .unwrap()
                                                    .wait()
                                                    .unwrap();
                                            }
                                        } else {
                                        edit::edit_file(temp_file_edit.path()).unwrap();
                                        }
                                        temp_file_edit.close().unwrap();
                                        std::process::exit(0);
                                    } else {
                                        println!("Cannot watch the file at \"{:?}\" for hot-reload.", temp_file);
                                        std::process::exit(104);
                                    }
                                } else {
                                    println!("Cannot use filesystem watcher for hot-reload.");
                                    // TODO: don't exit here?
                                    std::process::exit(103);
                                }
                            } else {
                                println!("could not write file \"{:?}\".", temp_file);
                                std::process::exit(102);
                            }
                        }
                        _ => (String::new(), String::new()),
                    };
                    parse::file::File::new(
                        contents,
                        path.into()
                    )
                }
                else if execute {
                    parse::file::File::new(
                        args.iter().skip(1).fold(String::new(), |mut s, v| {
                            if !s.is_empty() {
                                s.push(' ');
                            }
                            s.push_str(v);
                            s
                        }),
                        path.into(),
                    )
                } else {
                    println!("please provide either a file or -e and a script to run!");
                    std::process::exit(101);
                }
            } else {
                parse::file::File::new(std::fs::read_to_string(&args[0]).unwrap(), path.into())
            }
        }
    })
    .unwrap();
    println!(" - - - - -");
    let start = Instant::now();
    let out = script.run(std::env::args().skip(2).collect());
    let elapsed = start.elapsed();
    println!(" - - - - -");
    println!("Output ({}s)\n{out}", elapsed.as_secs_f64());
}
