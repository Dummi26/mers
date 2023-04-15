use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::libs;

use super::{
    block::RStatement,
    val_data::{VData, VDataEnum, VDataThreadEnum},
    val_type::{VSingleType, VType},
};

const EV_ERR: usize = 0;
// const EV_??? = 1;
pub const EVS: [&'static str; 1] = ["Err"];

#[derive(Clone, Debug)]
pub enum BuiltinFunction {
    // core
    Assume1, // assume []/[t] is [t], return t. Optionally provide a reason as to why (2nd arg)
    AssumeNoEnum, // assume enum(*)/t is t.
    NoEnum,
    Matches,
    // print
    Print,
    Println,
    Debug,
    // stdin
    StdinReadLine,
    // format
    ToString,
    Format,
    // parse
    ParseInt,
    ParseFloat,
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
    // Math
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Gt,
    Lt,
    Gtoe,
    Ltoe,
    Min,
    Max,
    // List
    Push,
    Insert,
    Pop,
    Remove,
    Get,
    Len,
    // String
    Contains,
    StartsWith,
    EndsWith,
    Trim,
    Substring,
    Regex,
}

impl BuiltinFunction {
    pub fn get(s: &str) -> Option<Self> {
        Some(match s {
            "assume1" => Self::Assume1,
            "assume_no_enum" => Self::AssumeNoEnum,
            "noenum" => Self::NoEnum,
            "matches" => Self::Matches,
            "print" => Self::Print,
            "println" => Self::Println,
            "debug" => Self::Debug,
            "read_line" => Self::StdinReadLine,
            "to_string" => Self::ToString,
            "format" => Self::Format,
            "parse_int" => Self::ParseInt,
            "parse_float" => Self::ParseFloat,
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
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "mod" => Self::Mod,
            "pow" => Self::Pow,
            "eq" => Self::Eq,
            "lt" => Self::Lt,
            "gt" => Self::Gt,
            "ltoe" => Self::Ltoe,
            "gtoe" => Self::Gtoe,
            "min" => Self::Min,
            "max" => Self::Max,
            "push" => Self::Push,
            "insert" => Self::Insert,
            "pop" => Self::Pop,
            "remove" => Self::Remove,
            "get" => Self::Get,
            "len" => Self::Len,
            "contains" => Self::Contains,
            "starts_with" => Self::StartsWith,
            "ends_with" => Self::EndsWith,
            "trim" => Self::Trim,
            "substring" => Self::Substring,
            "regex" => Self::Regex,
            _ => return None,
        })
    }
    pub fn can_take(&self, input: &Vec<VType>) -> bool {
        match self {
            Self::Assume1 => {
                if input.len() >= 1 {
                    let mut len0 = false;
                    let mut len1 = false;
                    for t in input[0].types.iter() {
                        match t {
                            VSingleType::Tuple(v) => match v.len() {
                                0 => len0 = true,
                                1 => len1 = true,
                                _ => return false,
                            },
                            _ => len1 = true,
                        }
                    }
                    if !len0 {
                        eprintln!("Warn: calling assume1 on a value of type {}, which will never be a length-0 tuple and therefore cannot fail.", input[0]);
                    }
                    if !len1 {
                        eprintln!("Warn: calling assume1 on a value of type {}, which will always be a length-0 tuple!", input[0]);
                    }
                    if input.len() >= 2 {
                        if input.len() == 2 {
                            input[1].fits_in(&VSingleType::String.to()).is_empty()
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Self::AssumeNoEnum => {
                if input.len() >= 1 {
                    let mut someenum = false;
                    let mut noenum = false;
                    for t in input[0].types.iter() {
                        match t {
                            VSingleType::EnumVariant(..) | VSingleType::EnumVariantS(..) => {
                                someenum = true
                            }
                            _ => noenum = true,
                        }
                    }
                    if !someenum {
                        eprintln!("Warn: calling assume_no_enum on a value of type {}, which will never be an enum and therefore cannot fail.", input[0]);
                    }
                    if !noenum {
                        eprintln!("Warn: calling assume_no_enum on a value of type {}, which will always be an enum!", input[0]);
                    }
                    if input.len() >= 2 {
                        if input.len() == 2 {
                            input[1].fits_in(&VSingleType::String.to()).is_empty()
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Self::NoEnum => input.len() == 1,
            Self::Matches => input.len() == 1,
            Self::Print | Self::Println => {
                if input.len() == 1 {
                    input[0].fits_in(&VSingleType::String.to()).is_empty()
                } else {
                    false
                }
            }
            Self::Debug => true,
            Self::ToString => true,
            Self::Format => {
                if let Some(format_string) = input.first() {
                    format_string.fits_in(&VSingleType::String.to()).is_empty()
                } else {
                    false
                }
            }
            Self::StdinReadLine => input.is_empty(),
            Self::ParseInt | Self::ParseFloat => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to()).is_empty()
            }
            Self::Run | Self::Thread => {
                if input.len() >= 1 {
                    input[0].types.iter().all(|v| {
                        if let VSingleType::Function(v) = v {
                            if v.iter().any(|(i, _)| i.len() == input.len() - 1) {
                                eprintln!("Warn: Function inputs aren't type checked yet!)");
                                // TODO!
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            }
            Self::Await => {
                input.len() == 1
                    && input[0]
                        .types
                        .iter()
                        .all(|v| matches!(v, VSingleType::Thread(_)))
            }
            Self::Sleep => {
                input.len() == 1
                    && input[0]
                        .fits_in(&VType {
                            types: vec![VSingleType::Int, VSingleType::Float],
                        })
                        .is_empty()
            }
            Self::Exit => {
                input.len() == 0
                    || (input.len() == 1 && input[0].fits_in(&VSingleType::Int.to()).is_empty())
            }
            // TODO!
            Self::FsList => true,
            Self::FsRead => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to()).is_empty()
            }
            Self::FsWrite => {
                input.len() == 2
                    && input[0].fits_in(&VSingleType::String.to()).is_empty()
                    && input[1]
                        .fits_in(&VSingleType::List(VSingleType::Int.to()).to())
                        .is_empty()
            }
            Self::BytesToString => {
                input.len() == 1
                    && input[0]
                        .fits_in(&VSingleType::List(VSingleType::Int.to()).to())
                        .is_empty()
            }
            Self::StringToBytes => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to()).is_empty()
            }
            Self::RunCommand | Self::RunCommandGetBytes => {
                if input.len() >= 1 && input[0].fits_in(&VSingleType::String.to()).is_empty() {
                    if input.len() == 1 {
                        true
                    } else if input.len() == 2 {
                        input[1]
                            .fits_in(&VSingleType::List(VSingleType::String.to()).to())
                            .is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Eq => input.len() == 2,
            Self::Add => {
                input.len() == 2 && {
                    let num = VType {
                        types: vec![VSingleType::Int, VSingleType::Float],
                    };
                    let st = VSingleType::String.to();
                    (input[0].fits_in(&num).is_empty() && input[1].fits_in(&num).is_empty())
                        || (input[0].fits_in(&st).is_empty() && input[1].fits_in(&st).is_empty())
                }
            }
            Self::Sub
            | Self::Mul
            | Self::Div
            | Self::Mod
            | Self::Pow
            | Self::Gt
            | Self::Lt
            | Self::Gtoe
            | Self::Ltoe
            | Self::Min
            | Self::Max => {
                input.len() == 2 && {
                    let num = VType {
                        types: vec![VSingleType::Int, VSingleType::Float],
                    };
                    input[0].fits_in(&num).is_empty() && input[1].fits_in(&num).is_empty()
                }
            }
            // TODO! check that we pass a reference to a list!
            Self::Push => {
                if input.len() == 2 {
                    // check if the element that should be inserted fits in the list's inner type
                    let (vec, el) = (&input[0], &input[1]);
                    // if vec.is_reference().is_some_and(|v| v) { // unstable
                    if let Some(true) = vec.is_reference() {
                        if let Some(t) = vec.get_any() {
                            el.fits_in(&t).is_empty()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Insert => {
                if input.len() == 3 {
                    let (vec, el) = (&input[0], &input[1]);
                    if let Some(true) = vec.is_reference() {
                        if let Some(t) = vec.get_any() {
                            el.fits_in(&t).is_empty()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Pop => {
                if input.len() == 1 {
                    let vec = &input[0];
                    if let Some(true) = vec.is_reference() {
                        // TODO! this also returns true for tuples. what should we do for tuples? should pop return (first_val rest_of_tuple) and not take a reference?
                        if let Some(_) = vec.get_any() {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Remove => {
                if input.len() == 2 {
                    let (vec, index) = (&input[0], &input[1]);
                    if let Some(true) = vec.is_reference() {
                        // TODO! same issue as in pop
                        if let Some(_) = vec.get_any() {
                            if index.fits_in(&VSingleType::Int.to()).is_empty() {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            // TODO! finish this
            Self::Get | Self::Len => true,
            Self::Substring => {
                if input.len() >= 2 && input.len() <= 3 {
                    let (s, start) = (&input[0], &input[1]);
                    let index_type = VSingleType::Int.to();
                    if s.fits_in(&VSingleType::String.to()).is_empty()
                        && start.fits_in(&index_type).is_empty()
                    {
                        if let Some(end) = input.get(2) {
                            end.fits_in(&index_type).is_empty()
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Contains | Self::StartsWith | Self::EndsWith | Self::Regex => {
                input.len() == 2
                    && input
                        .iter()
                        .all(|v| v.fits_in(&VSingleType::String.to()).is_empty())
            }
            Self::Trim => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to()).is_empty()
            }
        }
    }
    /// for invalid inputs, may panic
    pub fn returns(&self, input: Vec<VType>) -> VType {
        match self {
            Self::Assume1 => {
                let mut out = VType { types: vec![] };
                for t in &input[0].types {
                    match t {
                        VSingleType::Tuple(v) => {
                            if !v.is_empty() {
                                out = out | &v[0];
                            }
                        }
                        v => out = out | v.clone().to(),
                    }
                }
                out
            }
            Self::AssumeNoEnum => {
                let mut out = VType { types: vec![] };
                for t in &input[0].types {
                    match t {
                        VSingleType::EnumVariant(..) | VSingleType::EnumVariantS(..) => (),
                        t => out = out | t.clone().to(),
                    }
                }
                out
            }
            Self::NoEnum => input[0].clone().noenum(),
            Self::Matches => input[0].matches().1,
            // []
            Self::Print | Self::Println | Self::Debug | Self::Sleep => VType {
                types: vec![VSingleType::Tuple(vec![])],
            },
            Self::StdinReadLine => VSingleType::String.to(),
            // String
            Self::ToString | Self::Format => VSingleType::String.into(),
            Self::ParseInt => VType {
                types: vec![VSingleType::Tuple(vec![]), VSingleType::Int],
            },
            Self::ParseFloat => VType {
                types: vec![VSingleType::Tuple(vec![]), VSingleType::Float],
            },
            // !
            Self::Run | Self::Thread => {
                if let Some(funcs) = input.first() {
                    let mut out = VType { types: vec![] };
                    for func in &funcs.types {
                        if let VSingleType::Function(io) = func {
                            for (i, o) in io {
                                if i.iter()
                                    .zip(input.iter().skip(1))
                                    .all(|(i, input)| input.contains(i))
                                {
                                    out = out | o;
                                }
                            }
                        } else {
                            unreachable!("run called, first arg not a function")
                        }
                    }
                    match self {
                        Self::Run => out,
                        Self::Thread => VSingleType::Thread(out).to(),
                        _ => unreachable!(),
                    }
                } else {
                    unreachable!("run or thread called without args")
                }
            }
            Self::Await => {
                if let Some(v) = input.first() {
                    let mut out = VType { types: vec![] };
                    for v in &v.types {
                        if let VSingleType::Thread(v) = v {
                            out = out | v;
                        } else {
                            unreachable!("await called with non-thread arg")
                        }
                    }
                    out
                } else {
                    unreachable!("await called without args")
                }
            }
            Self::Pop | Self::Remove | Self::Get => {
                if let Some(v) = input.first() {
                    VType {
                        types: vec![
                            VSingleType::Tuple(vec![]),
                            VSingleType::Tuple(vec![v
                                .get_any()
                                .expect("cannot use get on this type")]),
                        ],
                    }
                } else {
                    unreachable!("get, pop or remove called without args")
                }
            }
            Self::Exit => VType { types: vec![] }, // doesn't return
            Self::FsList => VType {
                types: vec![
                    VSingleType::List(VSingleType::String.into()),
                    VSingleType::EnumVariant(EV_ERR, VSingleType::String.to()),
                ],
            },
            Self::FsRead => VType {
                types: vec![
                    VSingleType::List(VSingleType::Int.to()),
                    VSingleType::EnumVariant(EV_ERR, VSingleType::String.to()),
                ],
            },
            Self::FsWrite => VType {
                types: vec![
                    VSingleType::Tuple(vec![]).into(),
                    VSingleType::EnumVariant(EV_ERR, VSingleType::String.to()),
                ],
            },
            Self::BytesToString => VType {
                types: vec![
                    VSingleType::String,
                    VSingleType::EnumVariant(
                        EV_ERR,
                        VSingleType::Tuple(vec![
                            VSingleType::String.into(), // lossy string
                            VSingleType::String.into(), // error message
                        ])
                        .to(),
                    ),
                ],
            },
            Self::StringToBytes => VSingleType::List(VSingleType::Int.into()).into(),
            Self::RunCommand => VType {
                types: vec![
                    // error
                    VSingleType::EnumVariant(EV_ERR, VSingleType::String.to()),
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
                    VSingleType::EnumVariant(EV_ERR, VSingleType::String.to()),
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
            Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::Mod
            | Self::Pow
            | Self::Min
            | Self::Max => {
                if input.len() == 2 {
                    let mut might_be_string = false;
                    if let Self::Add = self {
                        match (
                            input[0].contains(&VSingleType::String),
                            input[1].contains(&VSingleType::String),
                        ) {
                            (true, true) => might_be_string = true,
                            (true, false) | (false, true) => unreachable!(),
                            (false, false) => (),
                        }
                    }
                    let o = match (
                        (
                            input[0].contains(&VSingleType::Int),
                            input[0].contains(&VSingleType::Float),
                        ),
                        (
                            input[1].contains(&VSingleType::Int),
                            input[1].contains(&VSingleType::Float),
                        ),
                    ) {
                        ((true, false), (true, false)) => VSingleType::Int.to(),
                        ((true, _), (true, _)) => VType {
                            types: vec![VSingleType::Int, VSingleType::Float],
                        },
                        ((false, false), (false, false)) => VType { types: vec![] },
                        _ => VSingleType::Float.to(),
                    };
                    if might_be_string {
                        o | VSingleType::String.to()
                    } else {
                        o
                    }
                } else {
                    unreachable!("called add/sub/mul/div/mod/pow with args != 2")
                }
            }
            Self::Eq | Self::Lt | Self::Gt | Self::Ltoe | Self::Gtoe => VSingleType::Bool.to(),
            Self::Push | Self::Insert => VSingleType::Tuple(vec![]).into(),
            Self::Len => VSingleType::Int.into(),
            Self::Contains | Self::StartsWith | Self::EndsWith => VSingleType::Bool.into(),
            Self::Trim => VSingleType::String.into(),
            Self::Substring => VSingleType::String.into(),
            Self::Regex => VType {
                types: vec![
                    // [string ...]
                    VSingleType::List(VSingleType::String.to()),
                    // Err(string)
                    VSingleType::EnumVariant(EV_ERR, VSingleType::String.to()),
                ],
            },
        }
    }
    pub fn run(
        &self,
        args: &Vec<RStatement>,
        vars: &Vec<Arc<Mutex<VData>>>,
        libs: &Arc<Vec<libs::Lib>>,
    ) -> VData {
        match self {
            Self::Assume1 => match args[0].run(vars, libs).data {
                VDataEnum::Tuple(mut v) => {
                    if let Some(v) = v.pop() {
                        v
                    } else {
                        panic!(
                            "ASSUMPTION FAILED: assume1 :: {}",
                            if args.len() > 1 {
                                if let VDataEnum::String(v) = args[1].run(vars, libs).data {
                                    v
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            },
                        );
                    }
                }
                v => v.to(),
            },
            Self::AssumeNoEnum => {
                let data = args[0].run(vars, libs);
                match data.data {
                    VDataEnum::EnumVariant(..) => panic!(
                        "ASSUMPTION FAILED: assume_no_enum :: found {} :: {}",
                        data,
                        if args.len() > 1 {
                            if let VDataEnum::String(v) = args[1].run(vars, libs).data {
                                v
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    ),
                    d => d.to(),
                }
            }
            Self::NoEnum => args[0].run(vars, libs).noenum(),
            Self::Matches => match args[0].run(vars, libs).data.matches() {
                Some(v) => VDataEnum::Tuple(vec![v]).to(),
                None => VDataEnum::Tuple(vec![]).to(),
            },
            BuiltinFunction::Print => {
                if let VDataEnum::String(arg) = args[0].run(vars, libs).data {
                    print!("{}", arg);
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!("print function called with non-string arg")
                }
            }
            BuiltinFunction::Println => {
                if let VDataEnum::String(arg) = args[0].run(vars, libs).data {
                    println!("{}", arg);
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Debug => {
                println!("{:#?}", args[0].run(vars, libs).data);
                VDataEnum::Tuple(vec![]).to()
            }
            Self::StdinReadLine => {
                let mut line = String::new();
                _ = std::io::stdin().read_line(&mut line);
                VDataEnum::String(line.trim_end_matches(['\n', '\r']).to_string()).to()
            }
            BuiltinFunction::ToString => {
                VDataEnum::String(format!("{}", args[0].run(vars, libs).data)).to()
            }
            BuiltinFunction::Format => {
                if let VDataEnum::String(mut text) = args.first().unwrap().run(vars, libs).data {
                    for (i, arg) in args.iter().skip(1).enumerate() {
                        text = text.replace(
                            &format!("{{{i}}}"),
                            &format!("{}", arg.run(vars, libs).data),
                        );
                    }
                    VDataEnum::String(text).to()
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::ParseInt => {
                if args.len() == 1 {
                    if let VDataEnum::String(s) = args[0].run(vars, libs).data {
                        if let Ok(s) = s.parse() {
                            VDataEnum::Int(s).to()
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    } else {
                        unreachable!("parse arg not string")
                    }
                } else {
                    unreachable!("parse args != 1")
                }
            }
            BuiltinFunction::ParseFloat => {
                if args.len() == 1 {
                    if let VDataEnum::String(s) = args[0].run(vars, libs).data {
                        if let Ok(s) = s.parse() {
                            VDataEnum::Float(s).to()
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    } else {
                        unreachable!("parse arg not string")
                    }
                } else {
                    unreachable!("parse args != 1")
                }
            }
            BuiltinFunction::Run => {
                if args.len() >= 1 {
                    if let VDataEnum::Function(f) = args[0].run(vars, libs).data {
                        if f.inputs.len() != args.len() - 1 {
                            unreachable!()
                        }
                        for (i, var) in f.inputs.iter().enumerate() {
                            let val = args[i + 1].run(vars, libs);
                            *vars[*var].lock().unwrap() = val;
                        }
                        f.run(vars, libs)
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            BuiltinFunction::Thread => {
                if args.len() >= 1 {
                    if let VDataEnum::Function(f) = args[0].run(vars, libs).data {
                        if f.inputs.len() != args.len() - 1 {
                            unreachable!()
                        }
                        // to prevent weird stuff from happening, the function args will be stored in different Arc<Mutex<_>>s. This means that the args are different for each thread, while any variables that are captured from outside will be shared.
                        let mut thread_vars = vars.clone();
                        let mut run_input_types = vec![];
                        for (i, var) in f.inputs.iter().enumerate() {
                            let val = args[i + 1].run(vars, libs);
                            run_input_types.push(val.out_single());
                            thread_vars[*var] = Arc::new(Mutex::new(val));
                        }
                        let out_type = f.out(&run_input_types);
                        let libs = libs.clone();
                        VDataEnum::Thread(
                            VDataThreadEnum::Running(std::thread::spawn(move || {
                                f.run(&thread_vars, &libs)
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
                    if let VDataEnum::Thread(t, _) = args[0].run(vars, libs).data {
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
                    match args[0].run(vars, libs).data {
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
                    if let VDataEnum::Int(v) = s.run(vars, libs).data {
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
                    if let VDataEnum::String(path) = args[0].run(vars, libs).data {
                        if args.len() > 1 {
                            eprintln!("NOT YET IMPLEMENTED (TODO!): fs_list advanced filters")
                        }
                        match std::fs::read_dir(path) {
                            Ok(entries) => VDataEnum::List(
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
                            .to(),
                            Err(e) => VDataEnum::EnumVariant(
                                EV_ERR,
                                Box::new(VDataEnum::String(e.to_string()).to()),
                            )
                            .to(),
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
                    if let VDataEnum::String(path) = args[0].run(vars, libs).data {
                        match std::fs::read(path) {
                            Ok(data) => VDataEnum::List(
                                VSingleType::Int.into(),
                                data.into_iter()
                                    .map(|v| VDataEnum::Int(v as _).to())
                                    .collect(),
                            )
                            .to(),
                            Err(e) => VDataEnum::EnumVariant(
                                EV_ERR,
                                Box::new(VDataEnum::String(e.to_string()).to()),
                            )
                            .to(),
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
                        (args[0].run(vars, libs).data, args[1].run(vars, libs).data)
                    {
                        if let Some(bytes) = vdata_to_bytes(&data) {
                            let file_path: PathBuf = path.into();
                            if let Some(p) = file_path.parent() {
                                _ = std::fs::create_dir_all(p);
                            }
                            match std::fs::write(file_path, bytes) {
                                Ok(_) => VDataEnum::Tuple(vec![]).to(),
                                Err(e) => VDataEnum::EnumVariant(
                                    EV_ERR,
                                    Box::new(VDataEnum::String(e.to_string()).to()),
                                )
                                .to(),
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
                    if let VDataEnum::List(_, byte_data) = args[0].run(vars, libs).data {
                        if let Some(bytes) = vdata_to_bytes(&byte_data) {
                            match String::from_utf8(bytes) {
                                Ok(v) => VDataEnum::String(v).to(),
                                Err(e) => {
                                    let err = e.to_string();
                                    VDataEnum::EnumVariant(
                                        EV_ERR,
                                        Box::new(
                                            VDataEnum::Tuple(vec![
                                                VDataEnum::String(
                                                    String::from_utf8_lossy(&e.into_bytes())
                                                        .into_owned(),
                                                )
                                                .to(),
                                                VDataEnum::String(err).to(),
                                            ])
                                            .to(),
                                        ),
                                    )
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
                    if let VDataEnum::String(s) = args[0].run(vars, libs).data {
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
                    if let VDataEnum::String(s) = args[0].run(vars, libs).data {
                        let mut command = std::process::Command::new(s);
                        if args.len() > 1 {
                            if let VDataEnum::List(_, args) = args[1].run(vars, libs).data {
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
                            Err(e) => VDataEnum::EnumVariant(
                                EV_ERR,
                                Box::new(VDataEnum::String(e.to_string()).to()),
                            )
                            .to(),
                        }
                    } else {
                        unreachable!("run_command not string arg")
                    }
                } else {
                    unreachable!("run_command not 1 arg")
                }
            }
            Self::Add => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::String(mut a), VDataEnum::String(b)) => {
                            a.push_str(b.as_str());
                            VDataEnum::String(a).to()
                        }
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a + b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a as f64 + b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a + b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a + b).to(),
                        _ => unreachable!("add: not a number/string"),
                    }
                } else {
                    unreachable!("add: not 2 args")
                }
            }
            Self::Sub => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a - b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a as f64 - b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a - b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a - b).to(),
                        _ => unreachable!("sub: not a number"),
                    }
                } else {
                    unreachable!("sub: not 2 args")
                }
            }
            Self::Mul => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a * b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a as f64 * b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a * b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a * b).to(),
                        _ => unreachable!("mul: not a number"),
                    }
                } else {
                    unreachable!("mul: not 2 args")
                }
            }
            Self::Div => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a / b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a as f64 / b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a / b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a / b).to(),
                        _ => unreachable!("div: not a number"),
                    }
                } else {
                    unreachable!("div: not 2 args")
                }
            }
            Self::Mod => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a % b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a as f64 % b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a % b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a % b).to(),
                        _ => unreachable!("mod: not a number"),
                    }
                } else {
                    unreachable!("mod: not 2 args")
                }
            }
            Self::Pow => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(if b == 0 {
                            1
                        } else if b > 0 {
                            a.pow(b as _)
                        } else {
                            0
                        })
                        .to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float((a as f64).powf(b)).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a.powi(b as _)).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a.powf(b)).to()
                        }
                        _ => unreachable!("pow: not a number"),
                    }
                } else {
                    unreachable!("pow: not 2 args")
                }
            }
            Self::Eq => {
                if args.len() == 2 {
                    VDataEnum::Bool(args[0].run(vars, libs) == args[1].run(vars, libs)).to()
                } else {
                    unreachable!("eq: not 2 args")
                }
            }
            Self::Gt => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(a > b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Bool(a as f64 > b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Bool(a > b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(a > b).to(),
                        _ => unreachable!("gt: not a number"),
                    }
                } else {
                    unreachable!("gt: not 2 args")
                }
            }
            Self::Lt => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(a < b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Bool((a as f64) < b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Bool(a < b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(a < b).to(),
                        _ => unreachable!("lt: not a number"),
                    }
                } else {
                    unreachable!("lt: not 2 args")
                }
            }
            Self::Gtoe => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(a >= b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Bool(a as f64 >= b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Bool(a >= b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(a >= b).to(),
                        _ => unreachable!("gtoe: not a number"),
                    }
                } else {
                    unreachable!("gtoe: not 2 args")
                }
            }
            Self::Ltoe => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(a <= b).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Bool(a as f64 <= b).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Bool(a <= b as f64).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(a <= b).to(),
                        _ => unreachable!("ltoe: not a number"),
                    }
                } else {
                    unreachable!("ltoe: not 2 args")
                }
            }
            Self::Min => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a.min(b)).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float((a as f64).min(b)).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a.min(b as f64)).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a.min(b)).to()
                        }
                        _ => unreachable!("min: not a number"),
                    }
                } else {
                    unreachable!("min: not 2 args")
                }
            }
            Self::Max => {
                if args.len() == 2 {
                    match (args[0].run(vars, libs).data, args[1].run(vars, libs).data) {
                        (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a.max(b)).to(),
                        (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float((a as f64).max(b)).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                            VDataEnum::Float(a.max(b as f64)).to()
                        }
                        (VDataEnum::Float(a), VDataEnum::Float(b)) => {
                            VDataEnum::Float(a.max(b)).to()
                        }
                        _ => unreachable!("max: not a number"),
                    }
                } else {
                    unreachable!("max: not 2 args")
                }
            }
            Self::Push => {
                if args.len() == 2 {
                    if let VDataEnum::Reference(v) = args[0].run(vars, libs).data {
                        if let VDataEnum::List(_, v) = &mut v.lock().unwrap().data {
                            v.push(args[1].run(vars, libs));
                        }
                        VDataEnum::Tuple(vec![]).to()
                    } else {
                        unreachable!("push: not a reference")
                    }
                } else {
                    unreachable!("push: not 2 args")
                }
            }
            Self::Insert => {
                if args.len() == 3 {
                    if let (VDataEnum::Reference(v), VDataEnum::Int(i)) =
                        (args[0].run(vars, libs).data, args[2].run(vars, libs).data)
                    {
                        if let VDataEnum::List(_, v) = &mut v.lock().unwrap().data {
                            v.insert(i as _, args[1].run(vars, libs));
                        }
                        VDataEnum::Tuple(vec![]).to()
                    } else {
                        unreachable!("insert: not a reference and index")
                    }
                } else {
                    unreachable!("insert: not 3 args")
                }
            }
            Self::Pop => {
                if args.len() == 1 {
                    if let VDataEnum::Reference(v) = args[0].run(vars, libs).data {
                        if let VDataEnum::List(_, v) = &mut v.lock().unwrap().data {
                            if let Some(v) = v.pop() {
                                VDataEnum::Tuple(vec![v])
                            } else {
                                VDataEnum::Tuple(vec![])
                            }
                            .to()
                        } else {
                            unreachable!("pop: not a list")
                        }
                    } else {
                        unreachable!("pop: not a reference")
                    }
                } else {
                    unreachable!("pop: not 1 arg")
                }
            }
            Self::Remove => {
                if args.len() == 2 {
                    if let (VDataEnum::Reference(v), VDataEnum::Int(i)) =
                        (args[0].run(vars, libs).data, args[1].run(vars, libs).data)
                    {
                        if let VDataEnum::List(_, v) = &mut v.lock().unwrap().data {
                            if v.len() > i as _ && i >= 0 {
                                let v = v.remove(i as _);
                                VDataEnum::Tuple(vec![v]).to()
                            } else {
                                VDataEnum::Tuple(vec![]).to()
                            }
                        } else {
                            unreachable!("remove: not a list")
                        }
                    } else {
                        unreachable!("remove: not a reference and index")
                    }
                } else {
                    unreachable!("remove: not 2 args")
                }
            }
            Self::Get => {
                if args.len() == 2 {
                    if let (container, VDataEnum::Int(i)) =
                        (args[0].run(vars, libs).data, args[1].run(vars, libs).data)
                    {
                        if i >= 0 {
                            match match container {
                                VDataEnum::Reference(v) => match &v.lock().unwrap().data {
                                    VDataEnum::List(_, v) | VDataEnum::Tuple(v) => {
                                        v.get(i as usize).map(|v| v.clone())
                                    }
                                    _ => unreachable!(
                                        "get: reference to something other than list/tuple"
                                    ),
                                },
                                VDataEnum::List(_, v) | VDataEnum::Tuple(v) => {
                                    v.get(i as usize).map(|v| v.clone())
                                }
                                _ => unreachable!("get: not a reference/list/tuple"),
                            } {
                                Some(v) => VDataEnum::Tuple(vec![v]).to(),
                                None => VDataEnum::Tuple(vec![]).to(),
                            }
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    } else {
                        unreachable!("get: not a list/tuple/reference and index")
                    }
                } else {
                    unreachable!("get: not 2 args")
                }
            }
            Self::Len => {
                if args.len() == 1 {
                    VDataEnum::Int(match args[0].run(vars, libs).data {
                        VDataEnum::String(v) => v.len(),
                        VDataEnum::Tuple(v) => v.len(),
                        VDataEnum::List(_, v) => v.len(),
                        _ => unreachable!("len: invalid type"),
                    } as _)
                    .to()
                } else {
                    unreachable!("len: not 1 arg")
                }
            }
            Self::Contains => {
                if args.len() == 2 {
                    if let VDataEnum::String(a1) = args[0].run(vars, libs).data {
                        if let VDataEnum::String(a2) = args[1].run(vars, libs).data {
                            VDataEnum::Bool(a1.contains(a2.as_str())).to()
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            Self::StartsWith => {
                if args.len() == 2 {
                    if let VDataEnum::String(a1) = args[0].run(vars, libs).data {
                        if let VDataEnum::String(a2) = args[1].run(vars, libs).data {
                            VDataEnum::Bool(a1.starts_with(a2.as_str())).to()
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            Self::EndsWith => {
                if args.len() == 2 {
                    if let VDataEnum::String(a1) = args[0].run(vars, libs).data {
                        if let VDataEnum::String(a2) = args[1].run(vars, libs).data {
                            VDataEnum::Bool(a1.ends_with(a2.as_str())).to()
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            Self::Trim => {
                if args.len() == 1 {
                    if let VDataEnum::String(a) = args[0].run(vars, libs).data {
                        VDataEnum::String(a.trim().to_string()).to()
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            Self::Substring => {
                if args.len() >= 2 {
                    if let VDataEnum::String(a) = args[0].run(vars, libs).data {
                        if args.len() > 3 {
                            unreachable!()
                        }
                        let left = if let VDataEnum::Int(left) = args[1].run(vars, libs).data {
                            left
                        } else {
                            unreachable!()
                        };
                        let len = if args.len() == 3 {
                            if let VDataEnum::Int(len) = args[2].run(vars, libs).data {
                                Some(len)
                            } else {
                                unreachable!()
                            }
                        } else {
                            None
                        };
                        let left = if left >= 0 { left as usize } else { 0 };
                        if let Some(len) = len {
                            if len >= 0 {
                                VDataEnum::String(
                                    a.chars()
                                        .skip(left)
                                        .take((len as usize).saturating_sub(left))
                                        .collect(),
                                )
                                .to()
                            } else {
                                // negative end index => max length
                                VDataEnum::String(
                                    a.chars().skip(left).take(len.abs() as usize).collect(),
                                )
                                .to()
                            }
                        } else {
                            VDataEnum::String(a.chars().skip(left).collect()).to()
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            Self::Regex => {
                if args.len() == 2 {
                    if let (VDataEnum::String(a), VDataEnum::String(regex)) =
                        (args[0].run(vars, libs).data, args[1].run(vars, libs).data)
                    {
                        match regex::Regex::new(regex.as_str()) {
                            Ok(regex) => VDataEnum::List(
                                VSingleType::String.to(),
                                regex
                                    .find_iter(a.as_str())
                                    .map(|v| VDataEnum::String(v.as_str().to_string()).to())
                                    .collect(),
                            )
                            .to(),
                            Err(e) => VDataEnum::EnumVariant(
                                EV_ERR,
                                Box::new(VDataEnum::String(e.to_string()).to()),
                            )
                            .to(),
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
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
