use std::{
    collections::HashMap,
    io::{BufRead, Stdin, StdinLock, Stdout, StdoutLock, Write},
};

use crate::lang::{val_data::VData, val_type::VType};

use super::{
    comms::{self, ByteData, ByteDataA, Message, MessageResponse, RespondableMessage},
    LibInitInfo, LibInitReq,
};

pub struct MyLib {
    // name: String,
    version: (u32, u32),
    // description: String,
    // functions: Vec<(String, Vec<VType>, VType)>,
    pub callbacks: Callbacks,
    enum_variants: Vec<(String, usize)>,
    stdin: StdinLock<'static>,
    stdin_no_lock: Stdin,
}
impl MyLib {
    pub fn new(
        name: String,
        version: (u32, u32),
        description: String,
        functions: Vec<(String, Vec<(Vec<VType>, VType)>)>,
    ) -> Self {
        let stdout_no_lock = std::io::stdout();
        let stdin_no_lock = std::io::stdin();
        let mut stdout = stdout_no_lock.lock();
        let mut stdin = stdin_no_lock.lock();
        // comms version
        stdout.write(1u128.as_byte_data_vec().as_slice()).unwrap();
        let init_req: LibInitReq = (version.0, version.1, name, description, functions);
        stdout
            .write(init_req.as_byte_data_vec().as_slice())
            .unwrap();
        stdout.flush();
        let enum_variants = LibInitInfo::from_byte_data(&mut stdin).unwrap();
        Self {
            // name: name.clone(),
            version,
            // description: description.clone(),
            // functions: functions.clone(),
            callbacks: Callbacks::empty(),
            enum_variants,
            stdin,
            stdin_no_lock,
        }
    }
    pub fn get_enums(&self) -> &Vec<(String, usize)> {
        &self.enum_variants
    }
    fn get_one_msg(&mut self) -> Result<Result<(), Message>, std::io::Error> {
        let id = u128::from_byte_data(&mut self.stdin)?;
        let message = Message::from_byte_data(&mut self.stdin)?;
        match message {
            Message::RunFunction(msg) => self.callbacks.run_function.run(Respondable::new(id, msg)),
        };
        Ok(Ok(()))
    }
    pub fn get_next_unhandled_message(&mut self) -> Result<(), Message> {
        loop {
            match self.get_one_msg() {
                Ok(Ok(())) => {}
                // unhandled message. return it to be handeled or included in the error
                Ok(Err(msg)) => return Err(msg),
                // i/o error, probably because mers exited. return successfully.
                Err(e) => return Ok(()),
            }
        }
    }
}

pub struct Respondable<M> {
    id: u128,
    pub msg: M,
}
impl<M> Respondable<M> {
    fn new(id: u128, msg: M) -> Self {
        Self { id, msg }
    }
}
impl<M> Respondable<M>
where
    M: RespondableMessage,
{
    pub fn respond(self, with: M::With) {
        let mut stdout = std::io::stdout().lock();
        stdout.write(&self.id.as_byte_data_vec()).unwrap();
        stdout
            .write(&self.msg.respond(with).as_byte_data_vec())
            .unwrap();
        stdout.flush().unwrap();
    }
}
impl<M> Respondable<M>
where
    M: Into<Message>,
{
    pub fn to_general(self) -> Respondable<Message> {
        Respondable::new(self.id, self.msg.into())
    }
}

pub struct Callbacks {
    pub run_function: Callback<comms::run_function::Message>,
}
impl Callbacks {
    pub fn empty() -> Self {
        Self {
            run_function: Callback::empty(),
        }
    }
}
pub struct Callback<M>
where
    M: super::comms::RespondableMessage,
{
    pub nonconsuming: Vec<Box<dyn FnMut(&M)>>,
    pub consuming: Option<Box<dyn FnMut(Respondable<M>)>>,
}
impl<M> Callback<M>
where
    M: super::comms::RespondableMessage,
{
    pub fn empty() -> Self {
        Self {
            nonconsuming: vec![],
            consuming: None,
        }
    }
    /// If the event was handled by a consuming function, returns Ok(r) where r is the returned value from the consuming function.
    /// If it wasn't handled (or only handled by nonconsuming functions), Err(m) is returned, giving ownership of the original message back to the caller for further handling.
    pub fn run(&mut self, msg: Respondable<M>) -> Result<(), Respondable<M>> {
        for f in self.nonconsuming.iter_mut() {
            f(&msg.msg);
        }
        if let Some(f) = self.consuming.as_mut() {
            Ok(f(msg))
        } else {
            Err(msg)
        }
    }
}
