use std::{path::PathBuf, thread::JoinHandle, time::Instant};

use crate::{
    parse::{self, parse::ScriptError},
    script::{block::RScript, val_data::VDataEnum},
};

mod base_comments;
mod base_return;
mod base_values;
mod menu;

pub fn start(spawn_new_terminal_for_editor: bool) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let (editor_join_handle, file_path) = crate::interactive_mode::fs_watcher::main(
        spawn_new_terminal_for_editor,
        "// Welcome to the mers tutor!

// This is an interactive experience. After making a change to this file,
// save and then reload it to see the tutor's updates.
// To begin, change the following value from false to true:

false
",
        Box::new(move |file| {
            let mut file =
                parse::file::File::new(std::fs::read_to_string(file).unwrap(), PathBuf::new());
            sender.send(parse::parse::parse(&mut file)).unwrap();
        }),
    )
    .unwrap();
    let mut tutor = Tutor {
        current_pos: 0,
        current_status: String::new(),
        written_status_byte_len: 0,
        editor_join_handle,
        file_path,
        receiver,
    };
    loop {
        if let VDataEnum::Bool(true) = tutor.let_user_make_change().run(vec![]).data {
            break;
        }
    }
    menu::run(tutor);
}

use menu::MAX_POS;

pub struct Tutor {
    current_pos: usize,
    current_status: String,
    written_status_byte_len: usize,
    editor_join_handle: JoinHandle<()>,
    file_path: PathBuf,
    receiver: std::sync::mpsc::Receiver<Result<RScript, ScriptError>>,
}
impl Tutor {
    /// only returns after a successful compile. before returning, does not call self.update() - you have to do that manually.
    pub fn let_user_make_change(&mut self) -> RScript {
        // eprintln!(" - - - - - - - - - - - - - - - - - - - - - - - - -");
        let script = loop {
            match self.receiver.recv().unwrap() {
                Err(e) => {
                    self.current_status = format!(
                        " - Error during build{}",
                        e.to_string()
                            .lines()
                            .map(|v| format!("\n// {v}"))
                            .collect::<String>()
                    )
                }
                Ok(script) => {
                    break script;
                }
            }
            self.update(None);
        };
        self.current_status = format!(" - OK");
        script
    }
    pub fn set_status(&mut self, new_status: String) {
        self.current_status = new_status;
    }
    pub fn update(&mut self, overwrite_contents_with: Option<&str>) {
        if self.editor_join_handle.is_finished() {
            eprintln!("Error has closed!");
            std::process::exit(0);
        }
        let string = std::fs::read_to_string(self.file_path()).unwrap();
        let status = format!(
            "// Tutor: {}/{MAX_POS}{}\n",
            self.current_pos, self.current_status,
        );
        let status_len = status.len();
        std::fs::write(
            self.file_path(),
            if let Some(new_content) = overwrite_contents_with {
                status + new_content
            } else {
                status + &string[self.written_status_byte_len..]
            },
        )
        .unwrap();
        self.written_status_byte_len = status_len;
        // ignore this update to the file
        _ = self.receiver.recv().unwrap();
    }
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}
