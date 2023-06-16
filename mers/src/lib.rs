mod inlib;
mod lang;
mod libs;
mod parsing;
mod pathutil;

pub use inlib::MyLib;
pub use lang::{fmtgs, global_info::GlobalScriptInfo, val_data::*, val_type::*};
pub use libs::comms::{ByteData, ByteDataA, Message, RespondableMessage};
pub use parsing::{parse::*, *};

pub mod prelude {
    pub use super::{
        lang::{
            val_data::{VData, VDataEnum},
            val_type::{VSingleType, VType},
        },
        MyLib, RespondableMessage,
    };
}
