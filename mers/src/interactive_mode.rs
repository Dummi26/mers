/// creates a temporary file, then opens it in the user's default editor. watches the fs for changes to the file and acts upon them.
pub mod fs_watcher {
    use notify::Watcher;
    use std::{
        fs,
        path::PathBuf,
        thread::{self, JoinHandle},
    };

    #[derive(Debug)]
    pub struct Error(String);

    /// on each file change, recompiles and runs the code. lets people experiment with mers without having to put a file anywhere
    pub fn playground(spawn_new_terminal_for_editor: bool) -> Result<(), Error> {
        main(
            spawn_new_terminal_for_editor,
            "// Welcome to mers! (interactive mode)

// put your name here, then save the file to run the script.
your_name = \"\"
greeting = \"Hello, {0}!\".format(your_name)
println(greeting)
",
            Box::new(|temp_file: &PathBuf| {
                println!();
                if let Ok(file_contents) = fs::read_to_string(&temp_file) {
                    let mut file =
                        crate::parse::file::File::new(file_contents, temp_file.to_path_buf());
                    match crate::parse::parse::parse(&mut file) {
                        Ok(func) => {
                            println!(" - - - - -");
                            let output = func.run(vec![]);
                            println!(" - - - - -");
                            println!("{}", output);
                        }
                        Err(e) => println!("{}", e.0.with_file_and_gsinfo(&file, e.1.as_ref())),
                    }
                } else {
                    println!("can't read file at {:?}!", temp_file);
                    std::process::exit(105);
                }
            }),
        )?
        .0
        .join()
        .unwrap();
        Ok(())
    }

    /// when the file is changed, calls the provided closure with the file's path.
    /// returns a JoinHandle which will finish when the user closes the editor and the file that is being used.
    pub fn main(
        spawn_new_terminal_for_editor: bool,
        initial_file_contents: &str,
        mut on_file_change: Box<dyn FnMut(&PathBuf) + Send>,
    ) -> Result<(JoinHandle<()>, PathBuf), Error> {
        let temp_file_edit = edit::Builder::new().suffix(".mers").tempfile().unwrap();
        let temp_file = temp_file_edit.path().to_path_buf();
        eprintln!(
            "Using temporary file at {temp_file:?}. Save the file to update the output here."
        );
        if let Ok(_) = std::fs::write(&temp_file, initial_file_contents) {
            if let Ok(mut watcher) = {
                let temp_file = temp_file.clone();
                // the file watcher
                notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = event {
                        match &event.kind {
                            notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                                on_file_change(&temp_file);
                            }
                            _ => (),
                        }
                    }
                })
            } {
                if let Ok(_) = watcher.watch(&temp_file, notify::RecursiveMode::NonRecursive) {
                    let out = if spawn_new_terminal_for_editor {
                        if let Ok(term) = std::env::var("TERM") {
                            let editor = edit::get_editor().unwrap();
                            eprintln!("launching \"{term} -e {editor:?} {temp_file:?}...");
                            let mut editor = std::process::Command::new(term)
                                .arg("-e")
                                .arg(&editor)
                                .arg(&temp_file)
                                .spawn()
                                .unwrap();
                            (
                                thread::spawn(move || {
                                    // wait for the editor to finish
                                    editor.wait().unwrap();
                                    // stop the watcher (this is absolutely necessary because it also moves the watcher into the closure,
                                    // which prevents it from being dropped too early)
                                    drop(watcher);
                                    // close and remove the temporary file
                                    temp_file_edit.close().unwrap();
                                }),
                                temp_file,
                            )
                        } else {
                            return Err(Error(format!("TERM environment variable not set.")));
                        }
                    } else {
                        let tf = temp_file.clone();
                        (
                            thread::spawn(move || {
                                edit::edit_file(temp_file).unwrap();
                                drop(watcher);
                                temp_file_edit.close().unwrap();
                            }),
                            tf,
                        )
                    };
                    Ok(out)
                } else {
                    return Err(Error(format!(
                        "Cannot watch the file at \"{:?}\" for hot-reload.",
                        temp_file
                    )));
                }
            } else {
                return Err(Error(format!(
                    "Cannot use filesystem watcher for hot-reload."
                )));
            }
        } else {
            return Err(Error(format!("could not write file \"{:?}\".", temp_file)));
        }
    }
}
