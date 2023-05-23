use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::libs;

use super::{
    code_runnable::RStatement,
    global_info::{GSInfo, GlobalScriptInfo},
    val_data::{thread::VDataThreadEnum, VData, VDataEnum},
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
    Clone,
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
    // Bool
    Not,
    And,
    Or,
    // Math
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
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
    IndexOf,
    Trim,
    Substring,
    Replace,
    Regex,
}

impl BuiltinFunction {
    pub fn get(s: &str) -> Option<Self> {
        Some(match s {
            "assume1" => Self::Assume1,
            "assume_no_enum" => Self::AssumeNoEnum,
            "noenum" => Self::NoEnum,
            "matches" => Self::Matches,
            "clone" => Self::Clone,
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
            "not" => Self::Not,
            "and" => Self::And,
            "or" => Self::Or,
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "mod" => Self::Mod,
            "pow" => Self::Pow,
            "eq" => Self::Eq,
            "ne" => Self::Ne,
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
            "index_of" => Self::IndexOf,
            "trim" => Self::Trim,
            "substring" => Self::Substring,
            "replace" => Self::Replace,
            "regex" => Self::Regex,
            _ => return None,
        })
    }
    pub fn can_take(&self, input: &Vec<VType>, info: &GlobalScriptInfo) -> bool {
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
                            input[1].fits_in(&VSingleType::String.to(), info).is_empty()
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
                            input[1].fits_in(&VSingleType::String.to(), info).is_empty()
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
            Self::Clone => input.len() == 1 && matches!(input[0].is_reference(), Some(true)),
            Self::Print | Self::Println => {
                if input.len() == 1 {
                    input[0].fits_in(&VSingleType::String.to(), info).is_empty()
                } else {
                    false
                }
            }
            Self::Debug => true,
            Self::ToString => true,
            Self::Format => {
                !input.is_empty()
                    && input
                        .iter()
                        .all(|v| v.fits_in(&VSingleType::String.to(), info).is_empty())
            }
            Self::StdinReadLine => input.is_empty(),
            Self::ParseInt | Self::ParseFloat => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to(), info).is_empty()
            }
            Self::Run | Self::Thread => {
                if input.len() >= 1 {
                    input[0].types.iter().all(|v| {
                        // all possible types of the input function must be function types
                        if let VSingleType::Function(v) = v {
                            // and all those functions must take as many inputs as were supplied to run() or thread() minus one (the function itself).
                            if v.iter()
                                .all(|(fn_in, _fn_out)| fn_in.len() == input.len() - 1)
                            {
                                eprintln!("Warn: Function inputs aren't type checked yet!)");
                                // all functions have the correct length, now check their types:
                                // this is more difficult than it seems, because if a function covers all input types on the first and second argument, that doesn't necessarily mean that it covers all possible cases:
                                // say out function is of type fn((int string []) (string int) []).
                                // this covers int/string for the first two arguments, but the function actually can't handle two ints or two strings as arguments, it requires exactly one int and one string.
                                // the most obvious implementation here would be a recursive function that goes over each type in the first argument, then calls itself recursively to check the second element and so on,
                                // but this would likely become slower than it should for complex functions.
                                // because of this, we just trust the programmer not to provide wrong arguments to run() and thread() for now,
                                // until a better solution is found.
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
                        .fits_in(
                            &VType {
                                types: vec![VSingleType::Int, VSingleType::Float],
                            },
                            info,
                        )
                        .is_empty()
            }
            Self::Exit => {
                input.len() == 0
                    || (input.len() == 1
                        && input[0].fits_in(&VSingleType::Int.to(), info).is_empty())
            }
            // TODO!
            Self::FsList => true,
            Self::FsRead => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to(), info).is_empty()
            }
            Self::FsWrite => {
                input.len() == 2
                    && input[0].fits_in(&VSingleType::String.to(), info).is_empty()
                    && input[1]
                        .fits_in(&VSingleType::List(VSingleType::Int.to()).to(), info)
                        .is_empty()
            }
            Self::BytesToString => {
                input.len() == 1
                    && input[0]
                        .fits_in(&VSingleType::List(VSingleType::Int.to()).to(), info)
                        .is_empty()
            }
            Self::StringToBytes => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to(), info).is_empty()
            }
            Self::RunCommand | Self::RunCommandGetBytes => {
                if input.len() >= 1 && input[0].fits_in(&VSingleType::String.to(), info).is_empty()
                {
                    if input.len() == 1 {
                        true
                    } else if input.len() == 2 {
                        input[1]
                            .fits_in(&VSingleType::List(VSingleType::String.to()).to(), info)
                            .is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Self::Eq | Self::Ne => input.len() == 2,
            Self::Add => {
                input.len() == 2 && {
                    let num = VType {
                        types: vec![VSingleType::Int, VSingleType::Float],
                    };
                    let st = VSingleType::String.to();
                    (input[0].fits_in(&num, info).is_empty()
                        && input[1].fits_in(&num, info).is_empty())
                        || (input[0].fits_in(&st, info).is_empty()
                            && input[1].fits_in(&st, info).is_empty())
                }
            }
            Self::Not => {
                input.len() == 1 && input[0].fits_in(&VSingleType::Bool.to(), info).is_empty()
            }
            Self::And | Self::Or => {
                input.len() == 2
                    && input
                        .iter()
                        .all(|v| v.fits_in(&VSingleType::Bool.to(), info).is_empty())
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
                    input[0].fits_in(&num, info).is_empty()
                        && input[1].fits_in(&num, info).is_empty()
                }
            }
            // TODO! check that we pass a reference to a list!
            Self::Push => {
                if input.len() == 2 {
                    // check if the element that should be inserted fits in the list's inner type
                    let (vec, el) = (&input[0], &input[1]);
                    // if vec.is_reference().is_some_and(|v| v) { // unstable
                    if let Some(true) = vec.is_reference() {
                        if let Some(t) = vec.get_any(info) {
                            el.fits_in(&t, info).is_empty()
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
                        if let Some(t) = vec.get_any(info) {
                            el.fits_in(&t, info).is_empty()
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
                        if let Some(_) = vec.get_any(info) {
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
                        if let Some(_) = vec.get_any(info) {
                            if index.fits_in(&VSingleType::Int.to(), info).is_empty() {
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
                    if s.fits_in(&VSingleType::String.to(), info).is_empty()
                        && start.fits_in(&index_type, info).is_empty()
                    {
                        if let Some(end) = input.get(2) {
                            end.fits_in(&index_type, info).is_empty()
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
            // two strings
            Self::Contains | Self::StartsWith | Self::EndsWith | Self::Regex => {
                input.len() == 2
                    && input
                        .iter()
                        .all(|v| v.fits_in(&VSingleType::String.to(), info).is_empty())
            }
            // two strings or &strings
            Self::IndexOf => {
                input.len() == 2
                    && input.iter().all(|v| {
                        v.fits_in(
                            &VType {
                                types: vec![
                                    VSingleType::String,
                                    VSingleType::Reference(Box::new(VSingleType::String)),
                                ],
                            },
                            info,
                        )
                        .is_empty()
                    })
            }
            Self::Replace => {
                input.len() == 3
                    && input
                        .iter()
                        .all(|v| v.fits_in(&VSingleType::String.to(), info).is_empty())
            }
            Self::Trim => {
                input.len() == 1 && input[0].fits_in(&VSingleType::String.to(), info).is_empty()
            }
        }
    }
    /// for invalid inputs, may panic
    pub fn returns(&self, input: Vec<VType>, info: &GlobalScriptInfo) -> VType {
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
            Self::Clone => input[0]
                .dereference()
                .expect("type is a reference, so it can be dereferenced"),
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
                                    .all(|(i, input)| input.contains(i, info))
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
            Self::Pop => {
                if let Some(v) = input.first() {
                    if let Some(v) = v.dereference() {
                        VType {
                            types: vec![
                                VSingleType::Tuple(vec![]),
                                VSingleType::Tuple(vec![v
                                    .get_any(info)
                                    .expect("cannot use get on this type")]),
                            ],
                        }
                    } else {
                        unreachable!("pop called on a non-reference");
                    }
                } else {
                    unreachable!("pop called without args");
                }
            }
            Self::Remove => {
                if input[1].fits_in(&VSingleType::Int.to(), info).is_empty() {
                    if let Some(v) = input[0].dereference() {
                        VType {
                            types: vec![
                                VSingleType::Tuple(vec![]),
                                VSingleType::Tuple(vec![v
                                    .get_any(info)
                                    .expect("cannot use get on this type")]),
                            ],
                        }
                    } else {
                        unreachable!("remove called on a non-reference");
                    }
                } else {
                    unreachable!("remove called, but second arg not an int");
                }
            }
            Self::Get => {
                if let Some(v) = input.first() {
                    VType {
                        types: vec![
                            VSingleType::Tuple(vec![]),
                            VSingleType::Tuple(vec![v
                                .get_any(info)
                                .expect("cannot use get on this type")]),
                        ],
                    }
                } else {
                    unreachable!("get called without args")
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
            Self::Not | Self::And | Self::Or => VSingleType::Bool.to(),
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
                            input[0].contains(&VSingleType::String, info),
                            input[1].contains(&VSingleType::String, info),
                        ) {
                            (true, true) => might_be_string = true,
                            (true, false) | (false, true) => unreachable!(),
                            (false, false) => (),
                        }
                    }
                    let o = match (
                        (
                            input[0].contains(&VSingleType::Int, info),
                            input[0].contains(&VSingleType::Float, info),
                        ),
                        (
                            input[1].contains(&VSingleType::Int, info),
                            input[1].contains(&VSingleType::Float, info),
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
            Self::Eq | Self::Ne | Self::Lt | Self::Gt | Self::Ltoe | Self::Gtoe => {
                VSingleType::Bool.to()
            }
            Self::Push | Self::Insert => VSingleType::Tuple(vec![]).into(),
            Self::Len => VSingleType::Int.into(),
            Self::Contains | Self::StartsWith | Self::EndsWith => VSingleType::Bool.into(),
            Self::IndexOf => VType {
                types: vec![VSingleType::Tuple(vec![]), VSingleType::Int],
            },
            Self::Trim => VSingleType::String.into(),
            Self::Substring => VSingleType::String.into(),
            Self::Replace => VSingleType::String.to(),
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
    pub fn run(&self, args: &Vec<RStatement>, info: &GSInfo) -> VData {
        match self {
            Self::Assume1 => {
                let mut a0 = args[0].run(info);
                match a0.operate_on_data_immut(|v| {
                    if let VDataEnum::Tuple(v) = v {
                        if let Some(v) = v.get(0) {
                            Ok(Some(v.clone_data()))
                        } else {
                            Err(())
                        }
                    } else {
                        Ok(None)
                    }
                }) {
                    Ok(Some(v)) => v,
                    Ok(None) => a0,
                    Err(()) => {
                        let msg = if args.len() > 1 {
                            args[1].run(info).operate_on_data_immut(|v| {
                                if let VDataEnum::String(v) = v {
                                    Some(v.to_owned())
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        };
                        if let Some(m) = msg {
                            panic!("ASSUMPTION FAILED: assume1 :: {m}");
                        } else {
                            panic!("ASSUMPTION FAILED: assume1");
                        }
                    }
                }
            }
            Self::AssumeNoEnum => {
                let a0 = args[0].run(info);
                let was_ok = a0.operate_on_data_immut(|v| match v {
                    VDataEnum::EnumVariant(..) => false,
                    _ => true,
                });
                if was_ok {
                    a0
                } else {
                    let msg = if args.len() > 1 {
                        args[1].run(info).operate_on_data_immut(|v| {
                            if let VDataEnum::String(v) = v {
                                Some(v.to_owned())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    };
                    panic!(
                        "ASSUMPTION FAILED: assume_no_enum :: found {}{}",
                        a0.gsi(info.clone()),
                        if let Some(m) = msg {
                            format!(" :: {m}")
                        } else {
                            String::new()
                        }
                    );
                }
            }
            Self::NoEnum => args[0].run(info).noenum(),
            Self::Matches => match args[0].run(info).matches() {
                Some(v) => VDataEnum::Tuple(vec![v]).to(),
                None => VDataEnum::Tuple(vec![]).to(),
            },
            Self::Clone => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::Reference(r) = v {
                    r.clone_data()
                } else {
                    unreachable!()
                }
            }),
            BuiltinFunction::Print => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(arg) = v {
                    #[cfg(not(feature = "nushell_plugin"))]
                    {
                        print!("{}", arg);
                        std::io::stdout().flush();
                    }
                    #[cfg(feature = "nushell_plugin")]
                    {
                        eprint!("{}", arg);
                        std::io::stderr().flush();
                    }
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!("print function called with non-string arg")
                }
            }),
            BuiltinFunction::Println => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(arg) = v {
                    #[cfg(not(feature = "nushell_plugin"))]
                    println!("{}", arg);
                    #[cfg(feature = "nushell_plugin")]
                    eprintln!("{}", arg);
                    VDataEnum::Tuple(vec![]).to()
                } else {
                    unreachable!()
                }
            }),
            BuiltinFunction::Debug => {
                let val = args[0].run(info);
                #[cfg(not(feature = "nushell_plugin"))]
                println!(
                    "{} :: {} :: {}",
                    args[0].out(info).gsi(info.clone()),
                    val.out().gsi(info.clone()),
                    val.gsi(info.clone())
                );
                #[cfg(feature = "nushell_plugin")]
                eprintln!(
                    "{} :: {} :: {}",
                    args[0].out(info).gsi(info.clone()),
                    val.out().gsi(info.clone()),
                    val.gsi(info.clone())
                );
                VDataEnum::Tuple(vec![]).to()
            }
            Self::StdinReadLine => {
                let mut line = String::new();
                _ = std::io::stdin().read_line(&mut line);
                VDataEnum::String(line.trim_end_matches(['\n', '\r']).to_string()).to()
            }
            BuiltinFunction::ToString => {
                VDataEnum::String(args[0].run(info).gsi(info.clone()).to_string()).to()
            }
            BuiltinFunction::Format => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(text) = v {
                    let mut text = text.to_owned();
                    for (i, arg) in args.iter().skip(1).enumerate() {
                        arg.run(info).operate_on_data_immut(|v| {
                            if let VDataEnum::String(v) = v {
                                text = text.replace(&format!("{{{i}}}"), v);
                            } else {
                                unreachable!()
                            }
                        })
                    }
                    VDataEnum::String(text).to()
                } else {
                    unreachable!()
                }
            }),
            BuiltinFunction::ParseInt => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(s) = v {
                    if let Ok(s) = s.parse() {
                        VDataEnum::Int(s).to()
                    } else {
                        VDataEnum::Tuple(vec![]).to()
                    }
                } else {
                    unreachable!("parse arg not string")
                }
            }),
            BuiltinFunction::ParseFloat => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(s) = v {
                    if let Ok(s) = s.parse() {
                        VDataEnum::Float(s).to()
                    } else {
                        VDataEnum::Tuple(vec![]).to()
                    }
                } else {
                    unreachable!("parse arg not string")
                }
            }),
            BuiltinFunction::Run => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::Function(f) = v {
                    if f.inputs.len() != args.len() - 1 {
                        unreachable!("wrong input count")
                    }
                    for (i, var) in f.inputs.iter().enumerate() {
                        let val = args[i + 1].run(info).clone_data();
                        *var.lock().unwrap() = val;
                    }
                    f.run(info)
                } else {
                    unreachable!()
                }
            }),
            BuiltinFunction::Thread => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::Function(f) = v {
                    if f.inputs.len() != args.len() - 1 {
                        unreachable!("wrong input count")
                    }
                    let mut run_input_types = vec![];
                    for (i, var) in f.inputs.iter().enumerate() {
                        let val = args[i + 1].run(info).clone_data();
                        run_input_types.push(val.out_single());
                        *var.lock().unwrap() = val;
                    }
                    let out_type = f.out(&run_input_types);
                    let info = Arc::clone(info);
                    let f = Arc::clone(f);
                    VDataEnum::Thread(
                        VDataThreadEnum::Running(std::thread::spawn(move || f.run(&info))).to(),
                        out_type,
                    )
                    .to()
                } else {
                    unreachable!()
                }
            }),
            BuiltinFunction::Await => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::Thread(t, _) = v {
                    t.get()
                } else {
                    unreachable!()
                }
            }),
            BuiltinFunction::Sleep => args[0].run(info).operate_on_data_immut(|v| {
                match v {
                    VDataEnum::Int(v) => std::thread::sleep(Duration::from_secs(*v as _)),
                    VDataEnum::Float(v) => std::thread::sleep(Duration::from_secs_f64(*v)),
                    _ => unreachable!(),
                }
                VDataEnum::Tuple(vec![]).to()
            }),
            Self::Exit => {
                if let Some(s) = args.first() {
                    let code = s.run(info).operate_on_data_immut(|v| {
                        if let VDataEnum::Int(v) = v {
                            *v
                        } else {
                            1
                        }
                    });
                    std::process::exit(code as _);
                } else {
                    std::process::exit(1);
                }
            }
            Self::FsList => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(path) = v {
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
            }),
            Self::FsRead => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(path) = v {
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
            }),
            Self::FsWrite => args[0].run(info).operate_on_data_immut(|path| {
                args[1].run(info).operate_on_data_immut(|bytes| {
                    if let (VDataEnum::String(path), VDataEnum::List(_, data)) = (path, bytes) {
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
                            unreachable!("fs_write data arg not a [int]")
                        }
                    } else {
                        unreachable!("fs_write wrong args")
                    }
                })
            }),
            Self::BytesToString => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::List(_, byte_data) = v {
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
            }),
            Self::StringToBytes => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::String(s) = v {
                    VDataEnum::List(
                        VSingleType::Int.into(),
                        s.bytes().map(|v| VDataEnum::Int(v as isize).to()).collect(),
                    )
                    .to()
                } else {
                    unreachable!("string_to_bytes arg not string")
                }
            }),
            Self::RunCommand | Self::RunCommandGetBytes => {
                args[0].run(info).operate_on_data_immut(|v| {
                    args[1].run(info).operate_on_data_immut(|v2| {
                        if let VDataEnum::String(s) = v {
                            let mut command = std::process::Command::new(s);
                            if args.len() > 1 {
                                if let VDataEnum::List(_, args) = v2 {
                                    for arg in args {
                                        arg.operate_on_data_immut(|v| {
                                            if let VDataEnum::String(v) = v {
                                                command.arg(v);
                                            } else {
                                                unreachable!("run_command second arg not [string].")
                                            }
                                        })
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
                    })
                })
            }
            Self::Not => args[0].run(info).operate_on_data_immut(|v| {
                if let VDataEnum::Bool(v) = v {
                    VDataEnum::Bool(!v).to()
                } else {
                    unreachable!()
                }
            }),
            Self::And => args[0].run(info).operate_on_data_immut(|a| {
                if let VDataEnum::Bool(a) = a {
                    if *a == false {
                        VDataEnum::Bool(false).to()
                    } else {
                        args[1].run(info).operate_on_data_immut(|b| {
                            if let VDataEnum::Bool(b) = b {
                                VDataEnum::Bool(*b).to()
                            } else {
                                unreachable!()
                            }
                        })
                    }
                } else {
                    unreachable!()
                }
            }),
            Self::Or => args[0].run(info).operate_on_data_immut(|a| {
                if let VDataEnum::Bool(a) = a {
                    if *a == true {
                        VDataEnum::Bool(true).to()
                    } else {
                        args[1].run(info).operate_on_data_immut(|b| {
                            if let VDataEnum::Bool(b) = b {
                                VDataEnum::Bool(*b).to()
                            } else {
                                unreachable!()
                            }
                        })
                    }
                } else {
                    unreachable!()
                }
            }),
            Self::Add => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::String(a), VDataEnum::String(b)) => {
                        VDataEnum::String(format!("{a}{b}")).to()
                    }
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a + b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float(*a as f64 + b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float(a + *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a + b).to(),
                    _ => unreachable!("add: not a number/string"),
                })
            }),
            Self::Sub => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a - b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float(*a as f64 - *b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float(*a - *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a - b).to(),
                    _ => unreachable!("sub: not a number"),
                })
            }),
            Self::Mul => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a * b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float(*a as f64 * b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float(a * *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a * b).to(),
                    _ => unreachable!("mul: not a number"),
                })
            }),
            Self::Div => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a / b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float(*a as f64 / b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float(a / *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a / b).to(),
                    _ => unreachable!("div: not a number"),
                })
            }),
            Self::Mod => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(a % b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float(*a as f64 % b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float(a % *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Float(a % b).to(),
                    _ => unreachable!("mod: not a number"),
                })
            }),
            Self::Pow => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int(if *b == 0 {
                        1
                    } else if *b > 0 {
                        (*a).pow(*b as _)
                    } else {
                        0
                    })
                    .to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float((*a as f64).powf(*b)).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float((*a).powi(*b as _)).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float((*a).powf(*b)).to()
                    }
                    _ => unreachable!("pow: not a number"),
                })
            }),
            Self::Eq => VDataEnum::Bool(args[0].run(info) == args[1].run(info)).to(),
            Self::Ne => VDataEnum::Bool(args[0].run(info) != args[1].run(info)).to(),
            Self::Gt => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(*a > *b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Bool(*a as f64 > *b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Bool(*a > *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(*a > *b).to(),
                    _ => unreachable!("gt: not a number"),
                })
            }),
            Self::Lt => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(*a < *b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Bool((*a as f64) < *b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Bool(*a < *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(*a < *b).to(),
                    _ => unreachable!("lt: not a number"),
                })
            }),
            Self::Gtoe => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(*a >= *b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Bool(*a as f64 >= *b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Bool(*a >= *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(*a >= *b).to(),
                    _ => unreachable!("gtoe: not a number"),
                })
            }),
            Self::Ltoe => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Bool(*a <= *b).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Bool(*a as f64 <= *b).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Bool(*a <= *b as f64).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => VDataEnum::Bool(*a <= *b).to(),
                    _ => unreachable!("ltoe: not a number"),
                })
            }),
            Self::Min => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int((*a).min(*b)).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float((*a as f64).min(*b)).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float((*a).min(*b as f64)).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float((*a).min(*b)).to()
                    }
                    _ => unreachable!("min: not a number"),
                })
            }),
            Self::Max => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| match (a, b) {
                    (VDataEnum::Int(a), VDataEnum::Int(b)) => VDataEnum::Int((*a).max(*b)).to(),
                    (VDataEnum::Int(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float((*a as f64).max(*b)).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Int(b)) => {
                        VDataEnum::Float((*a).max(*b as f64)).to()
                    }
                    (VDataEnum::Float(a), VDataEnum::Float(b)) => {
                        VDataEnum::Float((*a).max(*b)).to()
                    }
                    _ => unreachable!("max: not a number"),
                })
            }),
            Self::Push => args[0].run(info).operate_on_data_mut(|list| {
                if let VDataEnum::Reference(v) = list {
                    v.operate_on_data_mut(|list| {
                        if let VDataEnum::List(_, v) = list {
                            v.push(args[1].run(info));
                        }
                        VDataEnum::Tuple(vec![]).to()
                    })
                } else {
                    unreachable!("push: not a reference")
                }
            }),
            Self::Insert => args[0].run(info).operate_on_data_mut(|v| {
                args[1].run(info).operate_on_data_immut(|i| {
                    // TODO: find out why the fuck this helps
                    if let (VDataEnum::Reference(v), VDataEnum::Int(i)) = (v, i) {
                        v.operate_on_data_mut(|v| {
                            if let VDataEnum::List(_, v) = v {
                                v.insert(*i as _, args[2].run(info));
                            }
                        });
                        VDataEnum::Tuple(vec![]).to()
                    } else {
                        unreachable!("insert: not a reference and index")
                    }
                })
            }),
            Self::Pop => args[0].run(info).operate_on_data_mut(|v| {
                if let VDataEnum::Reference(v) = v {
                    v.operate_on_data_mut(|v| {
                        if let VDataEnum::List(_, v) = v {
                            if let Some(v) = v.pop() {
                                VDataEnum::Tuple(vec![v])
                            } else {
                                VDataEnum::Tuple(vec![])
                            }
                            .to()
                        } else {
                            unreachable!("pop: not a list")
                        }
                    })
                } else {
                    unreachable!("pop: not a reference")
                }
            }),
            Self::Remove => args[0].run(info).operate_on_data_mut(|v| {
                args[1].run(info).operate_on_data_immut(|i|
                    // this being a reference means we wont need to call make_mut() later, so a .as_ref() borrow is enough.
                    if let (VDataEnum::Reference(v), VDataEnum::Int(i)) = (v, i
                    ) {
                        v.operate_on_data_mut(|v| {
                        if let VDataEnum::List(_, v) = v {
                                if *i >= 0 && v.len() > *i as _ {
                                    let v = v.remove(*i as _);
                                    VDataEnum::Tuple(vec![v]).to()
                                } else {
                                    VDataEnum::Tuple(vec![]).to()
                                }
                            } else {
                                unreachable!("remove: not a list")
                        }})
                    } else {
                        unreachable!("remove: not a reference and index")
                    })
            }),
            Self::Get => args[0].run(info).operate_on_data_immut(|container| {
                args[1].run(info).operate_on_data_immut(|i| {
                    if let VDataEnum::Int(i) = i {
                        if *i >= 0 {
                            container.get(*i as _).map_or_else(
                                || VDataEnum::Tuple(vec![]).to(),
                                |v| VDataEnum::Tuple(vec![v]).to(),
                            )
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    } else {
                        unreachable!("get: not a list/tuple/reference and index")
                    }
                })
            }),
            Self::Len => {
                if args.len() == 1 {
                    VDataEnum::Int(args[0].run(info).operate_on_data_immut(|v| match v {
                        VDataEnum::String(v) => v.len(),
                        VDataEnum::Tuple(v) => v.len(),
                        VDataEnum::List(_, v) => v.len(),
                        _ => unreachable!("len: invalid type"),
                    }) as _)
                    .to()
                } else {
                    unreachable!("len: not 1 arg")
                }
            }
            Self::Contains => args[0].run(info).operate_on_data_immut(|a1| {
                args[1].run(info).operate_on_data_immut(|a2| {
                    if let VDataEnum::String(a1) = a1 {
                        if let VDataEnum::String(a2) = a2 {
                            VDataEnum::Bool(a1.contains(a2.as_str())).to()
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                })
            }),
            Self::StartsWith => args[0].run(info).operate_on_data_immut(|a1| {
                args[1].run(info).operate_on_data_immut(|a2| {
                    if let VDataEnum::String(a1) = a1 {
                        if let VDataEnum::String(a2) = a2 {
                            VDataEnum::Bool(a1.starts_with(a2.as_str())).to()
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                })
            }),
            Self::EndsWith => args[0].run(info).operate_on_data_immut(|a1| {
                args[1].run(info).operate_on_data_immut(|a2| {
                    if let VDataEnum::String(a1) = a1 {
                        if let VDataEnum::String(a2) = a2 {
                            VDataEnum::Bool(a1.ends_with(a2.as_str())).to()
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                })
            }),
            Self::IndexOf => args[0].run(info).operate_on_data_immut(|find_in| {
                args[1].run(info).operate_on_data_immut(|pat| {
                    fn find(find_in: &String, pat: &String) -> VData {
                        if let Some(found_byte_index) = find_in.find(pat) {
                            if let Some(char_index) = find_in.char_indices().enumerate().find_map(
                                |(char_index, (byte_index, _char))| {
                                    if byte_index == found_byte_index {
                                        Some(char_index)
                                    } else {
                                        None
                                    }
                                },
                            ) {
                                VDataEnum::Int(char_index as _).to()
                            } else {
                                VDataEnum::Tuple(vec![]).to()
                            }
                        } else {
                            VDataEnum::Tuple(vec![]).to()
                        }
                    }
                    let o = match (find_in, pat) {
                        (VDataEnum::String(a), VDataEnum::String(b)) => find(a, b),
                        _ => unreachable!(),
                    };
                    o
                })
            }),
            Self::Trim => args[0].run(info).operate_on_data_immut(|a| {
                if let VDataEnum::String(a) = a {
                    VDataEnum::String(a.trim().to_string()).to()
                } else {
                    unreachable!()
                }
            }),
            Self::Substring => args[0].run(info).operate_on_data_immut(|a| {
                if let VDataEnum::String(a) = a {
                    if args.len() > 3 {
                        unreachable!()
                    }
                    let left = args[1].run(info).operate_on_data_immut(|left| {
                        if let VDataEnum::Int(left) = left {
                            *left
                        } else {
                            unreachable!()
                        }
                    });
                    let len = if args.len() == 3 {
                        args[2].run(info).operate_on_data_immut(|len| {
                            if let VDataEnum::Int(len) = len {
                                Some(*len)
                            } else {
                                unreachable!()
                            }
                        })
                    } else {
                        None
                    };
                    let left = if left >= 0 {
                        left as usize
                    } else {
                        (a.len() - 1).saturating_sub(left.abs() as _)
                    };
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
            }),
            Self::Replace => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|b| {
                    args[2].run(info).operate_on_data_immut(|c| {
                        if let (VDataEnum::String(a), VDataEnum::String(b), VDataEnum::String(c)) =
                            (a, b, c)
                        {
                            VDataEnum::String(a.replace(b, c)).to()
                        } else {
                            unreachable!()
                        }
                    })
                })
            }),
            Self::Regex => args[0].run(info).operate_on_data_immut(|a| {
                args[1].run(info).operate_on_data_immut(|regex| {
                    if let (VDataEnum::String(a), VDataEnum::String(regex)) = (a, regex) {
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
                })
            }),
        }
    }
}

fn vdata_to_bytes(vd: &Vec<VData>) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(vd.len());
    for b in vd {
        let b = b.operate_on_data_immut(|b| {
            if let VDataEnum::Int(b) = b {
                Some(*b)
            } else {
                None
            }
        })?;
        bytes.push(if 0 <= b && b <= u8::MAX as isize {
            b as u8
        } else if b.is_negative() {
            0
        } else {
            u8::MAX
        });
    }
    Some(bytes)
}
