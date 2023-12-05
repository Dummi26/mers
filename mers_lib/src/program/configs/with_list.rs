use std::{
    fmt::Display,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    data::{self, Data, MersData, MersType, Type},
    parsing::{statements::to_string_literal, Source},
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// Adds a simple list type
    /// `List` can store a variable number of items
    /// `as_list: fn` turns a tuple into a list
    /// `push: fn` adds an element to a list
    /// `pop: fn` removes the last element from a list. returns (element) or ().
    /// `get_mut: fn` like get, but returns a reference to the object
    /// TODO!
    /// `remove_at: fn` [TODO]
    /// `insert_at: fn` [TODO]
    pub fn with_list(self) -> Self {
        // TODO: Type with generics
        self
            .add_type("List".to_string(),
            Err(Arc::new(|s, i| {
                let mut src = Source::new_from_string_raw(s.to_owned());
                let srca = Arc::new(src.clone());
                let t = crate::parsing::types::parse_type(&mut src, &srca)?;
                Ok(Arc::new(ListT(crate::parsing::types::type_from_parsed(&t, i)?)))})))
            .add_var("get_mut".to_string(), Data::new(data::function::Function {
                    info: Arc::new(program::run::Info::neverused()),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Arc::new(|a, _i| {
                            let mut out = Type::empty_tuple();
                            for t in a.types.iter() {
                                if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                                    if t.0.len() != 2 {
                                        return Err(format!(
                                            "get_mut: argument must be a 2-tuple `(&List<_>, Int)`."
                                        ).into());
                                    }
                                    if t.0[1].is_included_in_single(&data::int::IntT) {
                                        if let Some(t) = t.0[0].dereference() {
                                            for t in t.types.iter() {
                                                if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                                    out.add(Arc::new(data::reference::ReferenceT(t.0.clone())));
                                                } else {
                                                    return Err(format!(
                                                        "get_mut: first argument in tuple {t} isn't `&List<_>`."
                                                    ).into());
                                                }
                                            }
                                        } else {
                                            return Err(format!(
                                                "get_mut: first type in tuple {t} isn't a reference."
                                            ).into());
                                        }
                                    } else {
                                        return Err(format!(
                                            "get_mut: Second type in tuple {t} wasn't `Int`."
                                        ).into());
                                    }
                                } else {
                                    return Err(format!(
                                        "get_mut: argument must be a 2-tuple `(&List<_>, Int)`."
                                    ).into());
                                }
                            }
                            Ok(out)
                    }),
                    run: Arc::new(|a, _i| {
                        let t = a.get();
                        let t = t.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                        let i = t.0[1].get().as_any().downcast_ref::<data::int::Int>().unwrap().0.max(0) as usize;
                        let o = match t.0[0].get()
                            .as_any()
                            .downcast_ref::<data::reference::Reference>()
                            .unwrap()
                            .0
                            .read()
                            .unwrap()
                            .get()
                            .as_any()
                            .downcast_ref::<List>()
                            .unwrap().0.get(i)
                        {
                            Some(data) => {
                                Data::new(data::reference::Reference(Arc::clone(data)))
                            }
                            None => Data::empty_tuple(),
                        };
                        o
                    }),
                inner_statements: None,
            }))
            .add_var(
                "pop".to_string(),
                Data::new(data::function::Function {
                    info: Arc::new(program::run::Info::neverused()),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Arc::new(|a, _i| {
                        if let Some(a) = a.dereference() {
                            let mut out = Type::empty();
                            for t in a.types.iter() {
                                if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                    out.add(Arc::new(t.0.clone()));
                                } else {
                                    return Err(format!(
                                        "pop: found a reference to {t}, which is not a list"
                                    ).into());
                                }
                            }
                            Ok(Type::newm(vec![
                                Arc::new(Type::new(data::tuple::TupleT(vec![out]))),
                                Arc::new(Type::empty_tuple())
                            ]))
                        } else {
                            return Err(format!("pop: not a reference: {a}").into());
                        }
                    }),
                    run: Arc::new(|a, _i| {
                        match a
                            .get()
                            .as_any()
                            .downcast_ref::<data::reference::Reference>()
                            .unwrap()
                            .0
                            .write()
                            .unwrap()
                            .get_mut()
                            .mut_any()
                            .downcast_mut::<List>()
                            .unwrap()
                            .0
                            .pop()
                        {
                            Some(data) => Data::one_tuple(data.read().unwrap().clone()),
                            None => Data::empty_tuple(),
                        }
                    }),
                inner_statements: None,
                }),
            )
            .add_var(
                "push".to_string(),
                Data::new(data::function::Function {
                    info: Arc::new(program::run::Info::neverused()),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Arc::new(|a, _i| {
                        for t in a.types.iter() {
                            if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                                if t.0.len() != 2 {
                                    return Err(format!(
                                        "push: tuple must have length 2"
                                    ).into());
                                }
                                let a = &t.0[0];
                                let new = &t.0[1];
                                if let Some(a) = a.dereference() {
                                    for t in a.types.iter() {
                                        if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                            if !new.is_included_in(&t.0) {
                                                return Err(format!(
                                            "push: found a reference to {t}, which is a list which can't contain elements of type {new}"
                                        ).into());
                                            }
                                        } else {
                                            return Err(format!(
                                                    "push: found a reference to {t}, which is not a list"
                                            ).into());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "push: first element in tuple not a reference: {a}"
                                    ).into());
                                }
                            } else {
                                return Err(format!("push: not a tuple: {t}")
                                .into());
                            }
                        }
                        Ok(Type::empty_tuple())
                    }),
                    run: Arc::new(|a, _i| {
                        let tuple = a.get();
                        let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                        tuple.0[0]
                            .get()
                            .as_any()
                            .downcast_ref::<data::reference::Reference>()
                            .unwrap()
                            .0
                            .write()
                            .unwrap()
                            .get_mut()
                            .mut_any()
                            .downcast_mut::<List>()
                            .unwrap()
                            .0
                            .push(Arc::new(RwLock::new(tuple.0[1].clone())));
                            Data::empty_tuple()
                    }),
                inner_statements: None,
                }),
            )
            .add_var(
                "as_list".to_string(),
                Data::new(data::function::Function {
                    info: Arc::new(program::run::Info::neverused()),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Arc::new(|a, _i| {
                        if let Some(v) = a.iterable() {
                            Ok(Type::new(ListT(v)))
                        } else {
                            Err(format!(
                                "cannot iterate over type {a}"
                            ).into())
                        }
                    }),
                    run: Arc::new(|a, _i| {
                        if let Some(i) = a.get().iterable() {
                            Data::new(List(i.map(|v| Arc::new(RwLock::new(v))).collect()))
                        } else {
                            unreachable!("as_list called on non-iterable")
                        }
                    }),
                inner_statements: None,
                }),
            )
    }
}

#[derive(Clone, Debug)]
pub struct List(Vec<Arc<RwLock<Data>>>);
#[derive(Debug)]
pub struct ListT(Type);
impl MersData for List {
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.0.len() == self.0.len()
                && self
                    .0
                    .iter()
                    .zip(other.0.iter())
                    .all(|(s, o)| *s.read().unwrap() == *o.read().unwrap())
        } else {
            false
        }
    }
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        Some(Box::new(
            self.0
                .clone()
                .into_iter()
                .map(|v| v.read().unwrap().clone()),
        ))
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        let mut t = Type::empty();
        for el in &self.0 {
            t.add(Arc::new(el.read().unwrap().get().as_type()));
        }
        Type::new(ListT(t))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl MersType for ListT {
    fn iterable(&self) -> Option<Type> {
        Some(self.0.clone())
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|v| self.0.is_same_type_as(&v.0))
    }
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        target
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|v| self.0.is_included_in(&v.0))
    }
    fn subtypes(&self, acc: &mut Type) {
        for t in self.0.subtypes_type().types {
            acc.add(Arc::new(Self(Type::newm(vec![t]))));
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}
impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c.read().unwrap().get())?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
impl Display for ListT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "List<{}>", to_string_literal(&self.0.to_string(), '>'))?;
        Ok(())
    }
}
