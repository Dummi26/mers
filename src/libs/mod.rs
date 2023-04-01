use std::{
    io::{self, BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::{
    parse::{file::File, parse},
    script::{
        val_data::{VData, VDataEnum},
        val_type::VType,
    },
};

/*
Libraries are processes that communicate via stdout/stdin.
data in stdout is only expected after it was requested via stdin. ignoring this will likely cause issues.
requests in stdin can be identified via the first byte (ascii char) and end with a \n newline character.
the identifying ascii chars:
  i init
    reply format:
      two bytes, the first for major and the second for minor version number.
      the utf8-encoded name of the library followed by a newline
      the number of lines in the description (0 for no description) as a byte. (more than 255 lines aren't supported)
      a (optionally markdown-formatted [TODO!]) description of the library; all lines (including the last one) must end with a newline
      the things you would like to register; one line each unless explicitly stated otherwise; the first byte (char) decides what type to register:
        f a function: followed by the function signature, i.e. "my_func(string int/float [string]) string/[float int]"
        x end: indicates that you are done registering things
  I init 2
    TODO! (currently nothing)
    reply should be a single line (only the newline char). If there is data before the newline char, it will be reported as an error and the script will not run.
  f call a function:
    followed by the function id byte (0 <= id < #funcs; function ids are assigned in ascending order as they were registered in the reply to "i"
    and the data for each argument, in order.
    reply: the data for the returned value
  x exit
sending data: (all ints are encoded so that the most significant data is sent FIRST)
  the first byte (ascii char) identifies the type of data: (exceptions listed first: bools)
    b false
    B true
    1 int
    2 int as string
    5 float
    6 float as string
    " string (format: ", 64-bit unsigned int indicating string length, so many bytes utf-8)


*/

#[derive(Debug)]
pub struct Lib {
    process: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    pub registered_fns: Vec<(String, Vec<VType>, VType)>,
}
impl Lib {
    pub fn launch(mut exec: Command) -> Result<Self, LaunchError> {
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
            writeln!(stdin, "i").unwrap();
            let vernum = {
                let mut vernum = [0, 0];
                stdout.read_exact(&mut vernum).unwrap();
                (vernum[0], vernum[1])
            };
            let name = stdout.line().unwrap();
            let name = name.trim();
            eprintln!("- <<< ADDING LIB: {name} v{}.{} >>> -", vernum.0, vernum.1);
            let lines_in_desc = stdout.one_byte().unwrap();
            let mut lines_desc = Vec::with_capacity(lines_in_desc as _);
            for _ in 0..lines_in_desc {
                let line = stdout.line().unwrap();
                let line = line.trim_end_matches('\n');
                eprintln!("| {line}");
                lines_desc.push(line.to_string());
            }
            let mut registered_fns = vec![];
            loop {
                let line = stdout.line().unwrap();
                match line.chars().next() {
                    Some('f') => {
                        let (name, args) = line[1..]
                            .split_once('(')
                            .expect("function signature didn't include the ( character.");
                        let mut fn_signature = File::new(args.to_string());
                        let mut fn_in = vec![];
                        loop {
                            let t = parse::parse_type_adv(&mut fn_signature, true).unwrap();
                            fn_in.push(t.0);
                            if t.1 {
                                break;
                            }
                        }
                        let fn_out = parse::parse_type(&mut fn_signature).unwrap();
                        eprintln!("Registering function \"{name}\" with args \"{}\" and return type \"{fn_out}\"", &fn_in.iter().fold(String::new(), |mut s, v| { s.push_str(format!(" {}", v).as_str()); s })[1..]);
                        registered_fns.push((name.to_string(), fn_in, fn_out));
                    }
                    _ => break,
                }
            }
            Ok(Self {
                process: handle,
                stdin: Arc::new(Mutex::new(stdin)),
                stdout: Arc::new(Mutex::new(stdout)),
                registered_fns,
            })
        } else {
            return Err(LaunchError::NoStdio);
        }
    }

    pub fn run_fn(&self, fnid: usize, args: &Vec<VData>) -> VData {
        let mut stdin = self.stdin.lock().unwrap();
        let mut stdout = self.stdout.lock().unwrap();
        debug_assert!(args.len() == self.registered_fns[fnid].1.len());
        write!(stdin, "f").unwrap();
        stdin.write(&[fnid as _]).unwrap();
        for (_i, arg) in args.iter().enumerate() {
            data_to_bytes(arg, &mut stdin);
        }
        let o = data_from_bytes(&mut stdout).to();
        o
    }
}

#[derive(Debug)]
pub enum LaunchError {
    NoStdio,
    CouldNotSpawnProcess(io::Error),
}

trait DirectReader {
    fn line(&mut self) -> Result<String, io::Error>;
    fn one_byte(&mut self) -> Result<u8, io::Error>;
}
impl<T> DirectReader for T
where
    T: BufRead,
{
    fn line(&mut self) -> Result<String, io::Error> {
        let mut buf = String::new();
        self.read_line(&mut buf)?;
        Ok(buf)
    }
    fn one_byte(&mut self) -> Result<u8, io::Error> {
        let mut b = [0];
        self.read(&mut b)?;
        Ok(b[0])
    }
}

fn data_to_bytes(data: &VData, stdin: &mut ChildStdin) {
    match &data.data {
        VDataEnum::Bool(false) => write!(stdin, "b").unwrap(),
        VDataEnum::Bool(true) => write!(stdin, "B").unwrap(),
        VDataEnum::Int(_) => todo!(),
        VDataEnum::Float(_) => todo!("floats are not yet implemented for LibFunction calls."),
        VDataEnum::String(s) => {
            write!(stdin, "\"").unwrap();
            stdin.write(&(s.len() as u64).to_be_bytes()).unwrap();
            stdin.write(s.as_bytes()).unwrap();
        }
        VDataEnum::Tuple(_) => todo!(),
        VDataEnum::List(..) => todo!(),
        VDataEnum::Function(..) | VDataEnum::Reference(..) | VDataEnum::Thread(..) => {
            panic!("cannot use functions, references or threads in LibFunctions.")
        }
        VDataEnum::EnumVariant(..) => todo!(),
    }
    stdin.flush().unwrap();
}
fn data_from_bytes(stdout: &mut BufReader<ChildStdout>) -> VDataEnum {
    match stdout.one_byte().unwrap().into() {
        'b' => VDataEnum::Bool(false),
        'B' => VDataEnum::Bool(true),
        '1' | '2' | '5' | '6' => todo!(),
        '"' => {
            let mut len_bytes = 0u64;
            for _ in 0..8 {
                len_bytes <<= 8;
                len_bytes |= stdout.one_byte().unwrap() as u64;
            }
            let mut buf = Vec::with_capacity(len_bytes as _);
            for _ in 0..len_bytes {
                buf.push(stdout.one_byte().unwrap());
            }
            VDataEnum::String(String::from_utf8_lossy(&buf).into_owned())
        }
        _ => todo!(),
    }
}
