use super::Data;

pub fn assign(from: Data, target: &Data) {
    let mut target = target.get_mut();
    if let Some(r) = target
        .mut_any()
        .downcast_mut::<crate::data::reference::Reference>()
    {
        *r.0.get_mut() = from.get().clone();
    } else {
        todo!("assignment to non-reference")
    }
}
