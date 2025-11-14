use super::Data;

pub fn assign(from: &Data, target: &Data) {
    if let Some(r) = target
        .get()
        .as_any()
        .downcast_ref::<crate::data::reference::Reference>()
    {
        *r.write() = from.clone();
    } else if let (Some((from, from_ref)), Some(target)) = (
        from.get()
            .as_any()
            .downcast_ref::<crate::data::tuple::Tuple>()
            .map(|v| (v.clone(), false))
            .or_else(|| {
                from.get()
                    .as_any()
                    .downcast_ref::<crate::data::reference::Reference>()
                    .and_then(|r| {
                        r.read()
                            .get()
                            .as_any()
                            .downcast_ref::<crate::data::tuple::Tuple>()
                            .map(|v| (v.clone_refs(), true))
                    })
            }),
        target
            .get()
            .as_any()
            .downcast_ref::<crate::data::tuple::Tuple>(),
    ) {
        for (from, target) in from.0.into_iter().zip(target.0.iter()) {
            if from_ref {
                assign(&Data::new(from), &*target.read());
            } else {
                assign(&*from.read(), &*target.read());
            }
        }
    } else if let (Some((from, from_ref)), Some(target)) = (
        from.get()
            .as_any()
            .downcast_ref::<crate::data::object::Object>()
            .map(|v| (v.clone(), false))
            .or_else(|| {
                from.get()
                    .as_any()
                    .downcast_ref::<crate::data::reference::Reference>()
                    .and_then(|r| {
                        r.read()
                            .get()
                            .as_any()
                            .downcast_ref::<crate::data::object::Object>()
                            .map(|v| (v.clone_refs(), true))
                    })
            }),
        target
            .get()
            .as_any()
            .downcast_ref::<crate::data::object::Object>(),
    ) {
        for (field, target) in target.iter() {
            if from_ref {
                let from = from
                    .get_mut(*field)
                    .expect("type-checks should guarantee that from has every field of target");
                assign(&Data::new(from.clone_ref()), &*target.read());
            } else {
                let from = from
                    .get(*field)
                    .expect("type-checks should guarantee that from has every field of target");
                assign(&from, &*target.read());
            }
        }
    } else {
        unreachable!("invalid assignment")
    }
}
