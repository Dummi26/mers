use super::Data;

pub fn assign(from: &Data, target: &Data) {
    if let Some(r) = target
        .get()
        .as_any()
        .downcast_ref::<crate::data::reference::Reference>()
    {
        *r.0.write().unwrap() = from.clone();
    } else if let (Some(from), Some(target)) = (
        from.get()
            .as_any()
            .downcast_ref::<crate::data::tuple::Tuple>(),
        target
            .get()
            .as_any()
            .downcast_ref::<crate::data::tuple::Tuple>(),
    ) {
        for (from, target) in from.0.iter().zip(target.0.iter()) {
            assign(from, target);
        }
    } else if let (Some(from), Some(target)) = (
        from.get()
            .as_any()
            .downcast_ref::<crate::data::object::Object>(),
        target
            .get()
            .as_any()
            .downcast_ref::<crate::data::object::Object>(),
    ) {
        for (field, target) in target.iter() {
            let from = from
                .get(*field)
                .expect("type-checks should guarantee that from has every field of target");
            assign(from, target);
        }
    } else {
        unreachable!("invalid assignment")
    }
}
