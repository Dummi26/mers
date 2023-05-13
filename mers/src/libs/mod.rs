pub mod comms;
pub mod inlib;
pub mod path;

use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::{
    lang::{
        global_info::GlobalScriptInfo,
        val_data::{VData, VDataEnum},
        val_type::VType,
    },
    libs::comms::{ByteData, ByteDataA},
    parsing::{file::File, parse},
};

use self::comms::{MessageResponse, RespondableMessage};

// Libraries are processes that communicate via stdout/stdin.

#[derive(Debug)]
pub struct Lib {
    name: String,
    process: Child,
    current_id: Arc<Mutex<u128>>,
    stdin: Arc<Mutex<ChildStdin>>,
    // stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    task_sender: Arc<
        Mutex<std::sync::mpsc::Sender<(u128, Box<dyn FnOnce(&mut BufReader<ChildStdout>) + Send>)>>,
    >,
    pub registered_fns: Vec<(String, Vec<(Vec<VType>, VType)>)>,
}
impl Drop for Lib {
    fn drop(&mut self) {
        if self.process.try_wait().is_err() {
            if let Err(e) = self.process.kill() {
                eprint!(
                    "Warn: tried to kill lib process for library \"{}\", but failed: {e:?}",
                    self.name
                );
            }
        }
    }
}
/// Sent by the library to request initialization
/// ([ver_major], [ver_minor], [name], [desc], [registered_functions])
pub type LibInitReq<'a> = (
    u32,
    u32,
    String,
    String,
    Vec<(String, Vec<(Vec<VType>, VType)>)>,
);
/// Sent by mers to finish initializing a library.
/// [enum variants]
pub type LibInitInfo = Vec<(String, usize)>;
pub type LibInitInfoRef<'a> = Vec<(&'a String, &'a usize)>;

impl Lib {
    pub fn launch(
        mut exec: Command,
        enum_variants: &mut HashMap<String, usize>,
    ) -> Result<Self, LaunchError> {
        let mut handle = match exec
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(v) => v,
            Err(e) => return Err(LaunchError::CouldNotSpawnProcess(e)),
        };
        if let (Some(mut stdin), Some(stdout)) = (handle.stdin.take(), handle.stdout.take()) {
            let mut stdout = BufReader::new(stdout);
            let comms_version: u128 = ByteData::from_byte_data(&mut stdout).unwrap();
            assert_eq!(comms_version, 1);
            let (ver_major, ver_minor, name, description, mut registered_fns) =
                LibInitReq::from_byte_data(&mut stdout).unwrap();
            eprintln!("- <<< ADDING LIB: {name} v{ver_major}.{ver_minor} >>> -");
            for line in description.lines() {
                eprintln!("  | {line}");
            }
            for (name, _) in registered_fns.iter() {
                eprintln!("  fn {name}");
            }
            let mut ginfo = GlobalScriptInfo::default();
            for (_name, func) in registered_fns.iter_mut() {
                for (args, out) in func.iter_mut() {
                    for t in args.iter_mut() {
                        crate::lang::to_runnable::stypes(t, &mut ginfo);
                    }
                    crate::lang::to_runnable::stypes(out, &mut ginfo);
                }
            }
            for (name, id) in ginfo.enum_variants {
                if !enum_variants.contains_key(&name) {
                    enum_variants.insert(name, id);
                }
            }
            let si: LibInitInfoRef = enum_variants.iter().collect();
            stdin.write(si.as_byte_data_vec().as_slice());
            stdin.flush();
            let (task_sender, recv) = std::sync::mpsc::channel::<(
                u128,
                Box<dyn FnOnce(&mut BufReader<ChildStdout>) + Send>,
            )>();
            let stdout_reader = std::thread::spawn(move || {
                let dur = std::time::Duration::from_millis(20);
                let mut pending = HashMap::new();
                loop {
                    // read id from stdout
                    if let Ok(id) = u128::from_byte_data(&mut stdout) {
                        // update pending
                        loop {
                            if let Ok((id, sender)) = recv.try_recv() {
                                pending.insert(id, sender);
                            } else {
                                break;
                            }
                        }
                        // find task with that id
                        if let Some(sender) = pending.remove(&id) {
                            // call the callback function, which will handle the rest
                            sender(&mut stdout)
                        } else {
                            eprintln!("ID {id} not found! possible decode/encode error?");
                        }
                        std::thread::sleep(dur);
                    } else {
                        eprintln!(
                            "Library has exited, tasks pending: {}",
                            pending.iter().enumerate().fold(
                                String::new(),
                                |mut s, (i, (id, _))| if i == 0 {
                                    format!("{id}")
                                } else {
                                    s.push_str(format!(", {id}").as_str());
                                    s
                                }
                            )
                        );
                        break;
                    }
                }
            });
            Ok(Self {
                name,
                process: handle,
                stdin: Arc::new(Mutex::new(stdin)),
                // stdout: Arc::new(Mutex::new(stdout)),
                task_sender: Arc::new(Mutex::new(task_sender)),
                current_id: Arc::new(Mutex::new(0)),
                registered_fns,
            })
        } else {
            return Err(LaunchError::NoStdio);
        }
    }

    pub fn run_fn(&self, fnid: usize, args: Vec<VData>) -> VData {
        self.get_response(comms::run_function::Message {
            function_id: fnid as _,
            args,
        })
        .result
    }
    fn get_response<M>(&self, msg: M) -> M::Response
    where
        M: RespondableMessage,
        <M as comms::RespondableMessage>::Response: Send + 'static,
    {
        let recv = {
            let mut id = self.current_id.lock().unwrap();
            let mut stdin = self.stdin.lock().unwrap();
            let (sender, recv) = std::sync::mpsc::sync_channel(2);
            self.task_sender
                .lock()
                .unwrap()
                .send((
                    *id,
                    Box::new(move |stdout| {
                        sender
                            .send(ByteData::from_byte_data(stdout).unwrap())
                            .unwrap();
                    }),
                ))
                .unwrap();
            // id - type_id - message
            stdin.write(id.as_byte_data_vec().as_slice()).unwrap();
            stdin
                .write(msg.msgtype_id().as_byte_data_vec().as_slice())
                .unwrap();
            stdin.write(msg.as_byte_data_vec().as_slice()).unwrap();
            stdin.flush().unwrap();
            *id = id.wrapping_add(1);
            recv
        };
        recv.recv().unwrap()
    }
}

#[derive(Debug)]
pub enum LaunchError {
    NoStdio,
    CouldNotSpawnProcess(io::Error),
}
impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoStdio => write!(f, "couldn't get stdio (stdin/stdout) from child process."),
            Self::CouldNotSpawnProcess(e) => write!(f, "couldn't spawn child process: {e}."),
        }
    }
}
