#![allow(unused)]
#![allow(dead_code)]

mod lang;
mod libs;
mod parsing;

pub use lang::{global_info::GlobalScriptInfo, val_data::*, val_type::*};
pub use libs::{
    comms::{ByteData, ByteDataA, Message, RespondableMessage},
    inlib::MyLib,
};
pub use parsing::*;

pub mod prelude {
    pub use super::{
        lang::{
            val_data::{VData, VDataEnum},
            val_type::{VSingleType, VType},
        },
        MyLib, RespondableMessage,
    };
}
