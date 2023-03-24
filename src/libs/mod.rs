use std::{
    io,
    process::{Child, ChildStdin, ChildStdout, Command},
};

pub struct Lib {
    process: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}
impl Lib {
    pub fn launch(mut exec: Command) -> Result<Self, LaunchError> {
        let mut handle = match exec.spawn() {
            Ok(v) => v,
            Err(e) => return Err(LaunchError::CouldNotSpawnProcess(e)),
        };
        if let (Some(stdin), Some(stdout), stderr) = (
            handle.stdin.take(),
            handle.stdout.take(),
            handle.stderr.take(),
        ) {
            Ok(Self {
                process: handle,
                stdin,
                stdout,
            })
        } else {
            return Err(LaunchError::NoStdio);
        }
    }
}

pub enum LaunchError {
    NoStdio,
    CouldNotSpawnProcess(io::Error),
}
