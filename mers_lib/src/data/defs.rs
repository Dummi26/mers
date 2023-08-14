use crate::program::run::CheckError;

use super::{Data, MersType, Type};

pub fn assign(from: Data, target: &Data) {
    let target = target.get();
    if let Some(r) = target
        .as_any()
        .downcast_ref::<crate::data::reference::Reference>()
    {
        *r.0.lock().unwrap().get_mut() = from.get().clone();
    } else {
        todo!("assignment to non-reference")
    }
}
