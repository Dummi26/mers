use std::{
    fmt::Display,
    io::{BufRead, BufReader, Read},
    process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::{
    data::{self, Data, MersData, MersType, Type},
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// adds utilities to run commands installed on the system and get their output.
    /// `run_command: fn` runs a command with arguments.
    /// Args: (cmd, args) where cmd is a string and args is an Iterable over strings
    /// `RunCommandError` holds the error if the command can't be executed
    /// returns (int/(), string, string) on success (status code, stdout, stderr)
    pub fn with_command_running(self) -> Self {
        self
            .add_type("RunCommandError".to_owned(), Ok(Arc::new(RunCommandErrorT)))
            .add_type("ChildProcess".to_owned(), Ok(Arc::new(ChildProcessT)))
            .add_var(
            "run_command".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.types.iter().all(|t| t.as_any().downcast_ref::<data::tuple::TupleT>().is_some_and(|t| t.0.len() == 2 && t.0[0].is_included_in(&data::string::StringT) && t.0[1].iterable().is_some_and(|t| t.is_included_in(&data::string::StringT)))) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![
                                Type::newm(vec![Arc::new(data::int::IntT), Arc::new(data::bool::BoolT)]),
                                Type::new(data::string::StringT),
                                Type::new(data::string::StringT),
                            ])),
                            Arc::new(RunCommandErrorT)
                        ]))
                    } else {
                        return Err(format!("run_command called with invalid arguments (must be (String, Iter<String>))").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let cmd = a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                    let (cmd, args) = (&cmd.0[0], &cmd.0[1]);
                    let cmd = cmd.get();
                    let (cmd, args) = (
                        cmd.as_any().downcast_ref::<data::string::String>().unwrap(),
                        args.get().iterable().unwrap(),
                    );
                    match Command::new(&cmd.0)
                        .args(args.map(|v| v.get().to_string()))
                        .output()
                    {
                        Ok(output) => {
                            let status = if let Some(code) = output.status.code() {
                                Data::new(data::int::Int(code as _))
                            } else {
                                Data::new(data::bool::Bool(output.status.success()))
                            };
                            let stdout =
                                String::from_utf8_lossy(&output.stdout).into_owned();
                            let stderr =
                                String::from_utf8_lossy(&output.stderr).into_owned();
                            Data::new(data::tuple::Tuple(vec![
                                status,
                                Data::new(data::string::String(stdout)),
                                Data::new(data::string::String(stderr)),
                            ]))
                        }
                        Err(e) => Data::new(RunCommandError(e.to_string())),
                    }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "spawn_command".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.types.iter().all(|t| t.as_any().downcast_ref::<data::tuple::TupleT>().is_some_and(|t| t.0.len() == 2 && t.0[0].is_included_in(&data::string::StringT) && t.0[1].iterable().is_some_and(|t| t.is_included_in(&data::string::StringT)))) {
                        Ok(Type::newm(vec![
                            Arc::new(ChildProcessT),
                            Arc::new(RunCommandErrorT)
                        ]))
                    } else {
                        return Err(format!("spawn_command called with invalid arguments (must be (String, Iter<String>))").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let cmd = a.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                    let (cmd, args) = (&cmd.0[0], &cmd.0[1]);
                    let cmd = cmd.get();
                    let (cmd, args) = (
                        cmd.as_any().downcast_ref::<data::string::String>().unwrap(),
                        args.get().iterable().unwrap(),
                    );
                    match Command::new(&cmd.0)
                        .args(args.map(|v| v.get().to_string()))
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                    {
                        Ok(mut child) => {
                            let a = child.stdin.take().unwrap();
                            let b = BufReader::new(child.stdout.take().unwrap());
                            let c = BufReader::new(child.stderr.take().unwrap());
                            Data::new(ChildProcess(Arc::new(Mutex::new((child, a, b, c)))))
                        }
                        Err(e) => Data::new(RunCommandError(e.to_string())),
                    }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "childproc_exited".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&ChildProcessT) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::bool::BoolT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        return Err(format!("childproc_exited called on non-ChildProcess type {a}").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let child = a.as_any().downcast_ref::<ChildProcess>().unwrap();
                        let mut child = child.0.lock().unwrap();
                        match child.0.try_wait() {
                            Ok(Some(_)) => Data::one_tuple(Data::new(data::bool::Bool(true))),
                            Ok(None) => Data::one_tuple(Data::new(data::bool::Bool(false))),
                            Err(_) => Data::empty_tuple(),
                        }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "childproc_await".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&ChildProcessT) {
                        Ok(Type::newm(vec![
                            Arc::new(data::int::IntT),
                            Arc::new(data::bool::BoolT),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        return Err(format!("childproc_exited called on non-ChildProcess type {a}").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let child = a.as_any().downcast_ref::<ChildProcess>().unwrap();
                        let mut child = child.0.lock().unwrap();
                        match child.0.wait() {
                            Ok(s) => if let Some(s) = s.code() {
                                Data::new(data::int::Int(s as _))
                            } else {
                                Data::new(data::bool::Bool(s.success()))
                            }
                            Err(_) => Data::empty_tuple(),
                        }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "childproc_read_byte".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&ChildProcessT) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::int::IntT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        return Err(format!("childproc_read_byte called on non-ChildProcess type {a}").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let child = a.as_any().downcast_ref::<ChildProcess>().unwrap();
                        let mut child = child.0.lock().unwrap();
                        let mut buf = [0];
                        if child.2.read_exact(&mut buf).is_ok() {
                            Data::one_tuple(Data::new(data::int::Int(buf[0] as _)))
                        } else {
                            Data::empty_tuple()
                        }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "childproc_readerr_byte".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&ChildProcessT) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::int::IntT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        return Err(format!("childproc_readerr_byte called on non-ChildProcess type {a}").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let child = a.as_any().downcast_ref::<ChildProcess>().unwrap();
                        let mut child = child.0.lock().unwrap();
                        let mut buf = [0];
                        if child.3.read_exact(&mut buf).is_ok() {
                            Data::one_tuple(Data::new(data::int::Int(buf[0] as _)))
                        } else {
                            Data::empty_tuple()
                        }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "childproc_read_line".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&ChildProcessT) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::string::StringT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        return Err(format!("childproc_read_line called on non-ChildProcess type {a}").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let child = a.as_any().downcast_ref::<ChildProcess>().unwrap();
                        let mut child = child.0.lock().unwrap();
                        let mut buf = String::new();
                        if child.2.read_line(&mut buf).is_ok() {
                            Data::one_tuple(Data::new(data::string::String(buf)))
                        } else {
                            Data::empty_tuple()
                        }
                }),
                inner_statements: None,
            }),
        )
        .add_var(
            "childproc_readerr_line".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    if a.is_included_in(&ChildProcessT) {
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![Type::new(data::string::StringT)])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    } else {
                        return Err(format!("childproc_read_line called on non-ChildProcess type {a}").into());
                    }
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let child = a.as_any().downcast_ref::<ChildProcess>().unwrap();
                        let mut child = child.0.lock().unwrap();
                        let mut buf = String::new();
                        if child.3.read_line(&mut buf).is_ok() {
                            Data::one_tuple(Data::new(data::string::String(buf)))
                        } else {
                            Data::empty_tuple()
                        }
                }),
                inner_statements: None,
            }),
        )
    }
}

#[derive(Clone, Debug)]
pub struct ChildProcess(
    Arc<
        Mutex<(
            std::process::Child,
            ChildStdin,
            BufReader<ChildStdout>,
            BufReader<ChildStderr>,
        )>,
    >,
);
#[derive(Clone, Debug)]
pub struct ChildProcessT;
impl MersData for ChildProcess {
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        None
    }
    fn get(&self, _i: usize) -> Option<Data> {
        None
    }
    fn is_eq(&self, other: &dyn MersData) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| Arc::ptr_eq(&self.0, &other.0))
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        Type::new(ChildProcessT)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl Display for ChildProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ChildProcess>")
    }
}
impl MersType for ChildProcessT {
    fn iterable(&self) -> Option<Type> {
        None
    }
    fn get(&self) -> Option<Type> {
        None
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other.as_any().is::<Self>()
    }
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        target.as_any().is::<Self>()
    }
    fn subtypes(&self, acc: &mut Type) {
        acc.add(Arc::new(self.clone()));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl Display for ChildProcessT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChildProcess")
    }
}

#[derive(Clone, Debug)]
pub struct RunCommandError(String);
#[derive(Debug, Clone)]
pub struct RunCommandErrorT;
impl MersData for RunCommandError {
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.0 == self.0
        } else {
            false
        }
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> data::Type {
        Type::new(RunCommandErrorT)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl MersType for RunCommandErrorT {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other.as_any().downcast_ref::<Self>().is_some()
    }
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        self.is_same_type_as(target)
    }
    fn subtypes(&self, acc: &mut Type) {
        acc.add(Arc::new(self.clone()));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl Display for RunCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Display for RunCommandErrorT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RunCommandError")
    }
}
