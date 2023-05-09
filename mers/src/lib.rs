#![allow(unused)]
#![allow(dead_code)]

mod libs;
mod parsing;
mod script;

pub use libs::{
    comms::{ByteData, ByteDataA, Message, RespondableMessage},
    inlib::MyLib,
};
pub use parsing::*;
pub use script::{val_data::*, val_type::*};

pub mod prelude {
    pub use super::{
        script::{val_data::*, val_type::*},
        MyLib, RespondableMessage,
    };
}
