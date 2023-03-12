use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use super::{
    block::RStatement,
    value::{VData, VDataEnum, VDataThreadEnum, VSingleType, VType},
};

#[derive(Clone, Debug)]
pub enum BuiltinFunction {
    // print
    Print,
    Println,
    Debug,
    // format
    ToString,
    Format,
    // math and basic operators (not possible, need to be different for each type)
    // Add,
    // Sub,
    // Mul,
    // Div,
    // Mod,
    // functions
    Run,
    Thread,
    Await,
    Sleep,
    Exit,
    // FS
    FsList,
    FsRead,
    FsWrite,
    BytesToString,
    StringToBytes,
    // OS
    RunCommand,
    RunCommandGetBytes,
}

impl BuiltinFunction {
    pub fn get(s: &str) -> Option<Self> {
        Some(match s {
            "print" => Self::Print,
            "println" => Self::Println,
            "debug" => Self::Debug,
            "to_string" => Self::ToString,
            "format" => Self::Format,
            "run" => Self::Run,
            "thread" => Self::Thread,
            "await" => Self::Await,
            "sleep" => Self::Sleep,
            "exit" => Self::Exit,
            // "command" => Self::Command,
            "fs_list" => Self::FsList,
            "fs_read" => Self::FsRead,
            "fs_write" => Self::FsWrite,
            "bytes_to_string" => Self::BytesToString,
            "string_to_bytes" => Self::StringToBytes,
            "run_command" => Self::RunCommand,
            "run_command_get_bytes" => Self::RunCommandGetBytes,
            _ => return None,
        })
    }
    pub fn returns(&self) -> VType {
        match self {
            // []
            Self::Print | Self::Println | Self::Debug | Self::Sleep => VType {
                types: vec![VSingleType::Tuple(vec![])],
            },
            // String
            Self::ToString | Self::Format => VSingleType::String.into(),
            // !
            Self::Run | Self::Thread | Self::Await => {
                VType { types: vec![] } // TODO!
                                        // unreachable!("this has to be implemented somewhere else!")
            }
            Self::Exit => VType { types: vec![] }, // doesn't return
            Self::FsList => VType {
                types: vec![
                    VSingleType::Tuple(vec![]).into(),
                    VSingleType::List(VSingleType::String.into()).into(),
                ],
            },
            Self::FsRead => VType {
                types: vec![
                    VSingleType::Tuple(vec![]).into(),
                    VSingleType::List(VSingleType::Int.into()).into(),
                ],
            },
            Self::FsWrite => VType {
                types: vec![
                    VSingleType::Tuple(vec![]).into(),
                    VSingleType::List(VSingleType::String.into()).into(),
                ],
            },
            Self::BytesToString => VType {
                types: vec![
                    VSingleType::String.into(),
                    VSingleType::Tuple(vec![
                        VSingleType::String.into(), // lossy string
                        VSingleType::String.into(), // error message
                    ])
                    .into(),
                ],
            },
            Self::StringToBytes => VSingleType::List(VSingleType::Int.into()).into(),
            Self::RunCommand => VType {
                types: vec![
                    // error
                    VSingleType::String.into(),
                    // success: Option<ExitCode>, stdout, stderr
                    VSingleType::Tuple(vec![
                        VType {
                            types: vec![VSingleType::Tuple(vec![]).into(), VSingleType::Int.into()],
                        },
                        VSingleType::String.into(),
                        VSingleType::String.into(),
                    ]),
                ],
            },
            Self::RunCommandGetBytes => VType {
                types: vec![
                    // error
                    VSingleType::String.into(),
                    // success: Option<ExitCode>, stdout, stderr
                    VSingleType::Tuple(vec![
                        VType {
                            types: vec![VSingleType::Tuple(vec![]).into(), VSingleType::Int.into()],
                        },
                        VSingleType::List(VSingleType::Int.into()).into(),
                        VSingleType::List(VSingleType::Int.into()).into(),
                    ]),
                ],
            },
        }
    }
    pub fn run(&self, args: &Vec<RStatement>, vars: &Vec<Arc<Mutex<VData>>>) -> VData {
        match self {
            BuiltinFunction::Print => {
                if let VDataEnum::String(arg) = args[0].run(vars).data {
                    print!("{}", arg);
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!("print function called with non-string arg")
                }
            }
            BuiltinFunction::Println => {
                if let VDataEnum::String(arg) = args[0].run(vars).data {
                    println!("{}", arg);
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Debug => {
                println!("{:#?}", args[0].run(vars).data);
                VDataEnum::Tuple(vec![]).to()
            }
            BuiltinFunction::ToString => {
                VDataEnum::String(format!("{}", args[0].run(vars).data)).to()
            }
            BuiltinFunction::Format => {
                if let VDataEnum::String(mut text) = args.first().unwrap().run(vars).data {
                    for (i, arg) in args.iter().skip(1).enumerate() {
                        text =
                            text.replace(&format!("{{{i}}}"), &format!("{}", arg.run(vars).data));
                    }
                    VDataEnum::String(text).to()
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Run => {
                if args.len() >= 1 {
                    if let VDataEnum::Function(f) = args[0].run(vars).data {
                        if f.inputs.len() != args.len() - 1 {
                            unreachable!()
                        }
                        for (i, var) in f.inputs.iter().enumerate() {
                            let val = args[i + 1].run(vars);
                            *vars[*var].lock().unwrap() = val;
                        }
                        f.run(vars)
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Thread => {
                if args.len() >= 1 {
                    if let VDataEnum::Function(f) = args[0].run(vars).data {
                        if f.inputs.len() != args.len() - 1 {
                            unreachable!()
                        }
                        // to prevent weird stuff from happening, the function args will be stored in different Arc<Mutex<_>>s. This means that the args are different for each thread, while any variables that are captured from outside will be shared.
                        let mut thread_vars = vars.clone();
                        let mut run_input_types = vec![];
                        for (i, var) in f.inputs.iter().enumerate() {
                            let val = args[i + 1].run(vars);
                            run_input_types.push(val.out_single());
                            thread_vars[*var] = Arc::new(Mutex::new(val));
                        }
                        let out_type = f.out(&run_input_types);
                        VDataEnum::Thread(
                            VDataThreadEnum::Running(std::thread::spawn(move || {
                                f.run(&thread_vars)
                            }))
                            .to(),
                            out_type,
                        )
                        .to()
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Await => {
                if args.len() == 1 {
                    if let VDataEnum::Thread(t, _) = args[0].run(vars).data {
                        t.get()
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Sleep => {
                if args.len() == 1 {
                    match args[0].run(vars).data {
                        VDataEnum::Int(v) => std::thread::sleep(Duration::from_secs(v as _)),
                        VDataEnum::Float(v) => std::thread::sleep(Duration::from_secs_f64(v)),
                        _ => unreachable!(),
                    }
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!()
                }
            }
            Self::Exit => {
                if let Some(s) = args.first() {
                    if let VDataEnum::Int(v) = s.run(vars).data {
                        std::process::exit(v as _);
                    } else {
                        std::process::exit(1);
                    }
                } else {
                    std::process::exit(1);
                }
            }
            Self::FsList => {
                if args.len() > 0 {
                    if let VDataEnum::String(path) = args[0].run(vars).data {
                        if args.len() > 1 {
                            todo!("fs_list advanced filters")
                        }
                        if let Ok(entries) = std::fs::read_dir(path) {
                            VDataEnum::List(
                                VSingleType::String.into(),
                                entries
                                    .filter_map(|entry| {
                                        if let Ok(entry) = entry {
                                            Some(
                                                VDataEnum::String(
                                                    entry.path().to_string_lossy().into_owned(),
                                                )
                                                .to(),
                                            )
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            )
                            .to()
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    } else {
                        unreachable!("fs_list first arg not a string")
                    }
                } else {
                    unreachable!("fs_list without args")
                }
            }
            Self::FsRead => {
                if args.len() > 0 {
                    if let VDataEnum::String(path) = args[0].run(vars).data {
                        if let Ok(data) = std::fs::read(path) {
                            VDataEnum::List(
                                VSingleType::Int.into(),
                                data.into_iter()
                                    .map(|v| VDataEnum::Int(v as _).to())
                                    .collect(),
                            )
                            .to()
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    } else {
                        unreachable!("fs_read first arg not a string")
                    }
                } else {
                    unreachable!("fs_read without args")
                }
            }
            Self::FsWrite => {
                if args.len() > 1 {
                    if let (VDataEnum::String(path), VDataEnum::List(_, data)) =
                        (args[0].run(vars).data, args[1].run(vars).data)
                    {
                        if let Some(bytes) = vdata_to_bytes(&data) {
                            let file_path: PathBuf = path.into();
                            if let Some(p) = file_path.parent() {
                                _ = std::fs::create_dir_all(p);
                            }
                            match std::fs::write(file_path, bytes) {
                                Ok(_) => VDataEnum::Tuple(vec![]).to(),
                                Err(e) => VDataEnum::String(e.to_string()).to(),
                            }
                        } else {
                            unreachable!(
                                "fs_write first arg not a string or second arg not a [int]"
                            )
                        }
                    } else {
                        unreachable!("fs_write second arg not a [int]")
                    }
                } else {
                    unreachable!("fs_write without 2 args")
                }
            }
            Self::BytesToString => {
                if args.len() == 1 {
                    if let VDataEnum::List(_, byte_data) = args[0].run(vars).data {
                        if let Some(bytes) = vdata_to_bytes(&byte_data) {
                            match String::from_utf8(bytes) {
                                Ok(v) => VDataEnum::String(v).to(),
                                Err(e) => {
                                    let err = e.to_string();
                                    VDataEnum::Tuple(vec![
                                        VDataEnum::String(
                                            String::from_utf8_lossy(&e.into_bytes()).into_owned(),
                                        )
                                        .to(),
                                        VDataEnum::String(err).to(),
                                    ])
                                    .to()
                                }
                            }
                        } else {
                            unreachable!("bytes_to_string arg not [int]")
                        }
                    } else {
                        unreachable!("bytes_to_string first arg not [int]")
                    }
                } else {
                    unreachable!("bytes_to_string not 1 arg")
                }
            }
            Self::StringToBytes => {
                if args.len() == 1 {
                    if let VDataEnum::String(s) = args[0].run(vars).data {
                        VDataEnum::List(
                            VSingleType::Int.into(),
                            s.bytes().map(|v| VDataEnum::Int(v as isize).to()).collect(),
                        )
                        .to()
                    } else {
                        unreachable!("string_to_bytes arg not string")
                    }
                } else {
                    unreachable!("string_to_bytes not 1 arg")
                }
            }
            Self::RunCommand | Self::RunCommandGetBytes => {
                if args.len() > 0 {
                    if let VDataEnum::String(s) = args[0].run(vars).data {
                        let mut command = std::process::Command::new(s);
                        if args.len() > 1 {
                            if let VDataEnum::List(_, args) = args[1].run(vars).data {
                                for arg in args {
                                    if let VDataEnum::String(v) = arg.data {
                                        command.arg(v);
                                    } else {
                                        unreachable!("run_command second arg not [string].")
                                    }
                                }
                            } else {
                                unreachable!("run_command second arg not [string]")
                            }
                        }
                        match command.output() {
                            Ok(out) => VDataEnum::Tuple(vec![
                                if let Some(code) = out.status.code() {
                                    VDataEnum::Int(code as _)
                                } else {
                                    VDataEnum::Tuple(vec![])
                                }
                                .to(),
                                match self {
                                    Self::RunCommandGetBytes => VDataEnum::List(
                                        VSingleType::Int.into(),
                                        out.stdout
                                            .iter()
                                            .map(|v| VDataEnum::Int(*v as _).to())
                                            .collect(),
                                    ),
                                    _ => VDataEnum::String(
                                        String::from_utf8_lossy(&out.stdout).into_owned(),
                                    ),
                                }
                                .to(),
                                match self {
                                    Self::RunCommandGetBytes => VDataEnum::List(
                                        VSingleType::Int.into(),
                                        out.stderr
                                            .iter()
                                            .map(|v| VDataEnum::Int(*v as _).to())
                                            .collect(),
                                    ),
                                    _ => VDataEnum::String(
                                        String::from_utf8_lossy(&out.stderr).into_owned(),
                                    ),
                                }
                                .to(),
                            ])
                            .to(),
                            Err(e) => VDataEnum::String(e.to_string()).to(),
                        }
                    } else {
                        unreachable!("run_command not string arg")
                    }
                } else {
                    unreachable!("run_command not 1 arg")
                }
            }
        }
    }
}

fn vdata_to_bytes(vd: &Vec<VData>) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(vd.len());
    for b in vd {
        if let VDataEnum::Int(b) = b.data {
            bytes.push(if 0 <= b && b <= u8::MAX as isize {
                b as u8
            } else if b.is_negative() {
                0
            } else {
                u8::MAX
            });
        } else {
            return None;
        }
    }
    Some(bytes)
}
