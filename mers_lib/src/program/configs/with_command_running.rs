use std::{
    fmt::Display,
    process::Command,
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
    pub fn with_command_running(self) -> Self {
        self.add_var(
            "run_command".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new( CheckInfo::neverused())),
                out: Arc::new(|a, i| todo!()),
                run: Arc::new(|a, _i| {
                    if let Some(cmd) = a.get().as_any().downcast_ref::<data::tuple::Tuple>() {
                        if let (Some(cmd), Some(args)) = (cmd.get(0), cmd.get(1)) {
                            if let (Some(cmd), Some(args)) = (
                                cmd.get().as_any().downcast_ref::<data::string::String>(),
                                args.get().iterable(),
                            ) {
                                match Command::new(&cmd.0)
                                    .args(args.map(|v| v.get().to_string()))
                                    .output()
                                {
                                    Ok(output) => {
                                        let status = if let Some(code) = output.status.code() {
                                            Data::new(data::int::Int(code as _))
                                        } else {
                                            Data::empty_tuple()
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
                            } else {
                                unreachable!("run_command called with arguments other than (String, <Iterable>).")
                            }
                        } else {
                            unreachable!("run_command called with too few arguments")
                        }
                    } else {
                        unreachable!("run_command called with non-tuple argument")
                    }
                }),
            }),
        )
    }
}

#[derive(Clone, Debug)]
pub struct RunCommandError(String);
#[derive(Debug)]
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
        write!(f, "RunCommandError: {}", self.0)
    }
}
impl Display for RunCommandErrorT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RunCommandError")
    }
}
