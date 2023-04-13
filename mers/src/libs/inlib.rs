use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use crate::{
    libs::DirectReader,
    script::{val_data::VData, val_type::VType},
};

use super::{data_from_bytes, data_to_bytes};

pub struct MyLib {
    name: String,
    version: (u8, u8),
    description: String,
    functions: Vec<(String, Vec<VType>, VType)>,
    enum_variants: HashMap<String, usize>,
}
impl MyLib {
    pub fn new(
        name: String,
        version: (u8, u8),
        description: String,
        functions: Vec<(String, Vec<VType>, VType)>,
    ) -> (Self, MyLibTaskCompletion) {
        (
            Self {
                name,
                version,
                description,
                functions,
                enum_variants: HashMap::new(),
            },
            MyLibTaskCompletion { _priv: () },
        )
    }
    pub fn get_enum(&self, e: &str) -> Option<usize> {
        self.enum_variants.get(e).map(|v| *v)
    }
    pub fn run<I, O>(
        &mut self,
        run: MyLibTaskCompletion,
        stdin: &mut I,
        stdout: &mut O,
    ) -> MyLibTask
    where
        I: BufRead,
        O: Write,
    {
        drop(run);
        match match stdin.one_byte().unwrap().into() {
            'i' => {
                assert_eq!(stdin.one_byte().unwrap() as char, '\n');
                stdout.write(&[self.version.0, self.version.1]).unwrap();
                writeln!(stdout, "{}", self.name).unwrap();
                stdout
                    .write(&[self.description.split('\n').count() as _])
                    .unwrap();
                writeln!(stdout, "{}", self.description).unwrap();
                for func in self.functions.iter() {
                    writeln!(
                        stdout,
                        "f{}({}) {}",
                        func.0,
                        func.1
                            .iter()
                            .enumerate()
                            .map(|(i, v)| if i == 0 {
                                format!("{v}")
                            } else {
                                format!(" {v}")
                            })
                            .collect::<String>(),
                        func.2
                    )
                    .unwrap();
                }
                writeln!(stdout, "x").unwrap();
                None
            }
            'I' => {
                let mut line = String::new();
                loop {
                    line.clear();
                    stdin.read_line(&mut line).unwrap();
                    if let Some((task, args)) = line.split_once(' ') {
                        match task {
                            "set_enum_id" => {
                                let (enum_name, enum_id) = args.split_once(' ').unwrap();
                                let name = enum_name.trim().to_string();
                                let id = enum_id.trim().parse().unwrap();
                                self.enum_variants.insert(name.clone(), id);
                            }
                            _ => todo!(),
                        }
                    } else {
                        match line.trim_end() {
                            "init_finished" => break,
                            _ => unreachable!(),
                        }
                    }
                }
                Some(MyLibTask::FinishedInit(MyLibTaskCompletion { _priv: () }))
            }
            'f' => {
                let fnid = stdin.one_byte().unwrap() as usize;
                Some(MyLibTask::RunFunction(MyLibTaskRunFunction {
                    function: fnid,
                    args: self.functions[fnid]
                        .1
                        .iter()
                        .map(|_| data_from_bytes(stdin).to())
                        .collect(),
                }))
            }
            _ => None,
        } {
            Some(v) => v,
            None => MyLibTask::None(MyLibTaskCompletion { _priv: () }),
        }
    }
}

pub enum MyLibTask {
    None(MyLibTaskCompletion),
    FinishedInit(MyLibTaskCompletion),
    RunFunction(MyLibTaskRunFunction),
}
pub struct MyLibTaskRunFunction {
    pub function: usize,
    pub args: Vec<VData>,
}
impl MyLibTaskRunFunction {
    pub fn done<O>(self, o: &mut O, returns: VData) -> MyLibTaskCompletion
    where
        O: Write,
    {
        data_to_bytes(&returns, o);
        MyLibTaskCompletion { _priv: () }
    }
}
pub struct MyLibTaskCompletion {
    _priv: (),
}
