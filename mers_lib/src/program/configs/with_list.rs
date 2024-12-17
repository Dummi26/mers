use std::sync::{Arc, Mutex};

use crate::{
    data::{
        self,
        int::{IntT, INT_MAX},
        Data, MersData, MersType, MersTypeWInfo, Type,
    },
    errors::CheckError,
    info::DisplayInfo,
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
    /// `insert: fn` changes the element at the given index to a new value, then returns true (index <= len) or false (index > len)
    /// `replace: fn` replaces and returns the element at the given index with some new value (index < len) or returns `()` (index >= len)
    /// `remove: fn` removes and returns the element at the given index (index < len) or returns `()` (index >= len)
    pub fn with_list(self) -> Self {
        // TODO: Type with generics
        self
            .add_type("List".to_string(),
            Err(Arc::new(|s, i| {
                let mut src = Source::new_from_string_raw(s.to_owned());
                let srca = Arc::new(src.clone());
                let t = crate::parsing::types::parse_type(&mut src, &srca)?;
                Ok(Arc::new(Type::new(ListT(crate::parsing::types::type_from_parsed(&t, i)?))))})))
            .add_var(
                "pop",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, i| {
                        if let Some(a) = a.dereference() {
                            let mut out = Type::empty();
                            for t in a.types.iter() {
                                if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                    out.add_all(&t.0);
                                } else {
                                    return Err(format!(
                                        "pop: found a reference to {}, which is not a list", t.with_info(i)
                                    ).into());
                                }
                            }
                            Ok(Type::newm(vec![
                                Arc::new(data::tuple::TupleT(vec![out])),
                                Arc::new(data::tuple::TupleT(vec![]))
                            ]))
                        } else {
                            return Err(format!("pop: not a reference: {}", a.with_info(i)).into());
                        }
                    })),
                    run: Arc::new(|a, _i| {
                        Ok(match a
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
                            Some(data) => Data::one_tuple(data),
                            None => Data::empty_tuple(),
                        })
                    }),
                inner_statements: None,
                },
            )
            .add_var(
                "push",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, i| {
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
                                            "push: found a reference to {}, which is a list which can't contain elements of type {}", t.with_info(i), new.with_info(i)
                                        ).into());
                                            }
                                        } else {
                                            return Err(format!(
                                                    "push: found a reference to {}, which is not a list", t.with_info(i)
                                            ).into());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "push: first element in tuple not a reference: {}", a.with_info(i)
                                    ).into());
                                }
                            } else {
                                return Err(format!("push: not a tuple: {}", t.with_info(i))
                                .into());
                            }
                        }
                        Ok(Type::empty_tuple())
                    })),
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
                            .push(tuple.0[1].clone());
                            Ok(Data::empty_tuple())
                    }),
                inner_statements: None,
                },
            )
            .add_var(
                "insert",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, i| {
                        for t in a.types.iter() {
                            if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                                if t.0.len() != 3 {
                                    return Err(format!(
                                        "insert: tuple must have length 3"
                                    ).into());
                                }
                                let a = &t.0[0];
                                let index = &t.0[1];
                                let new = &t.0[2];
                                if !index.is_included_in_single(&IntT(0, INT_MAX)) {
                                    return Err(format!("insert: index should be an Int<0..>, but was {}", index.with_info(i)).into());
                                }
                                if let Some(a) = a.dereference() {
                                    for t in a.types.iter() {
                                        if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                            if !new.is_included_in(&t.0) {
                                                return Err(format!(
                                            "insert: found a reference to {}, which is a list which can't contain elements of type {}", t.with_info(i), new.with_info(i)
                                        ).into());
                                            }
                                        } else {
                                            return Err(format!(
                                                    "insert: found a reference to {}, which is not a list", t.with_info(i)
                                            ).into());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "insert: first element in tuple not a reference: {}", a.with_info(i)
                                    ).into());
                                }
                            } else {
                                return Err(format!("insert: not a tuple: {}", t.with_info(i))
                                .into());
                            }
                        }
                        Ok(data::bool::bool_type())
                    })),
                    run: Arc::new(|a, _i| {
                        let tuple = a.get();
                        let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                        let index = tuple.0[1].get().as_any().downcast_ref::<data::int::Int>().unwrap().0 as usize;
                            let list = tuple.0[0]
                            .get();
                            let mut list = list
                            .as_any()
                            .downcast_ref::<data::reference::Reference>()
                            .unwrap()
                            .0
                            .write().unwrap();
                            let mut list = list
                            .get_mut();
                            let list = list
                            .mut_any()
                            .downcast_mut::<List>()
                            .unwrap();
                        if index > list.0.len() {
                            Ok(Data::new(data::bool::Bool(false)))
                        } else {
                            list.0.insert(index, tuple.0[2].clone());
                            Ok(Data::new(data::bool::Bool(true)))
                        }
                    }),
                inner_statements: None,
                },
            )
            .add_var(
                "replace",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, i| {
                        let mut o = Type::empty();
                        for t in a.types.iter() {
                            if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                                if t.0.len() != 3 {
                                    return Err(format!(
                                        "replace: tuple must have length 3"
                                    ).into());
                                }
                                let a = &t.0[0];
                                let index = &t.0[1];
                                let new = &t.0[2];
                                if !index.is_included_in_single(&IntT(0, INT_MAX)) {
                                    return Err(format!("replace: index should be an Int<0..>, but was {}", index.with_info(i)).into());
                                }
                                if let Some(a) = a.dereference() {
                                    for t in a.types.iter() {
                                        if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                            if !new.is_included_in(&t.0) {
                                                return Err(format!(
                                            "replace: found a reference to {}, which is a list which can't contain elements of type {}", t.with_info(i), new.with_info(i)
                                        ).into());
                                            }
                                            o.add_all(&t.0);
                                        } else {
                                            return Err(format!(
                                                    "replace: found a reference to {}, which is not a list", t.with_info(i)
                                            ).into());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "replace: first element in tuple not a reference: {}", a.with_info(i)
                                    ).into());
                                }
                            } else {
                                return Err(format!("replace: not a tuple: {}", t.with_info(i))
                                .into());
                            }
                        }
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![o])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    })),
                    run: Arc::new(|a, _i| {
                        let tuple = a.get();
                        let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                        let index = tuple.0[1].get().as_any().downcast_ref::<data::int::Int>().unwrap().0 as usize;
                            let list = tuple.0[0]
                            .get();
                            let mut list = list
                            .as_any()
                            .downcast_ref::<data::reference::Reference>()
                            .unwrap()
                            .0
                            .write().unwrap();
                            let mut list = list
                            .get_mut();
                            let list = list
                            .mut_any()
                            .downcast_mut::<List>()
                            .unwrap();
                        if index < list.0.len() {
                            Ok(Data::one_tuple(std::mem::replace(&mut list.0[index], tuple.0[2].clone())))
                        } else {
                            Ok(Data::empty_tuple())
                        }
                    }),
                inner_statements: None,
                },
            )
            .add_var(
                "remove",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, i| {
                        let mut o = Type::empty();
                        for t in a.types.iter() {
                            if let Some(t) = t.as_any().downcast_ref::<data::tuple::TupleT>() {
                                if t.0.len() != 2 {
                                    return Err(format!(
                                        "remove: tuple must have length 2"
                                    ).into());
                                }
                                let a = &t.0[0];
                                let index = &t.0[1];
                                if !index.is_included_in_single(&IntT(0, INT_MAX)) {
                                    return Err(format!("remove: index should be an Int<0..>, but was {}", index.with_info(i)).into());
                                }
                                if let Some(a) = a.dereference() {
                                    for t in a.types.iter() {
                                        if let Some(t) = t.as_any().downcast_ref::<ListT>() {
                                            o.add_all(&t.0);
                                        } else {
                                            return Err(format!(
                                                    "remove: found a reference to {}, which is not a list", t.with_info(i)
                                            ).into());
                                        }
                                    }
                                } else {
                                    return Err(format!(
                                        "remove: first element in tuple not a reference: {}", a.with_info(i)
                                    ).into());
                                }
                            } else {
                                return Err(format!("remove: not a tuple: {}", t.with_info(i))
                                .into());
                            }
                        }
                        Ok(Type::newm(vec![
                            Arc::new(data::tuple::TupleT(vec![o])),
                            Arc::new(data::tuple::TupleT(vec![])),
                        ]))
                    })),
                    run: Arc::new(|a, _i| {
                        let tuple = a.get();
                        let tuple = tuple.as_any().downcast_ref::<data::tuple::Tuple>().unwrap();
                        let index = tuple.0[1].get().as_any().downcast_ref::<data::int::Int>().unwrap().0 as usize;
                            let list = tuple.0[0]
                            .get();
                            let mut list = list
                            .as_any()
                            .downcast_ref::<data::reference::Reference>()
                            .unwrap()
                            .0
                            .write().unwrap();
                            let mut list = list
                            .get_mut();
                            let list = list
                            .mut_any()
                            .downcast_mut::<List>()
                            .unwrap();
                        if index < list.0.len() {
                            Ok(Data::one_tuple(list.0.remove(index)))
                        } else {
                            Ok(Data::empty_tuple())
                        }
                    }),
                inner_statements: None,
                },
            )
            .add_var(
                "as_list",
                data::function::Function {
                    info: program::run::Info::neverused(),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Ok(Arc::new(|a, i| {
                        if let Some(v) = a.iterable() {
                            Ok(Type::new(ListT(v)))
                        } else {
                            Err(format!(
                                "cannot iterate over type {}", a.with_info(i)
                            ).into())
                        }
                    })),
                    run: Arc::new(|a, i| {
                        if let Some(iter) = a.get().iterable(&i.global) {
                            Ok(Data::new(List(iter.collect::<Result<_, _>>()?)))
                        } else {
                            Err("as_list called on non-iterable".into())
                        }
                    }),
                inner_statements: None,
                },
            )
    }
}

#[derive(Debug, Clone)]
pub struct List(pub Vec<Data>);
#[derive(Debug)]
pub struct ListT(pub Type);
impl MersData for List {
    fn display(&self, info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            c.get().display(info, f)?;
        }
        write!(f, "]")?;
        Ok(())
    }
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.0.len() == self.0.len()
                && self.0.iter().zip(other.0.iter()).all(|(s, o)| *s == *o)
        } else {
            false
        }
    }
    fn iterable(
        &self,
        _gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        Some(Box::new(self.0.clone().into_iter().map(|v| Ok(v))))
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        Type::new(ListT(self.inner_type()))
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
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(
            f,
            "List<{}>",
            to_string_literal(&self.0.with_display(info).to_string(), '>')
        )
    }
    fn iterable(&self) -> Option<Type> {
        Some(self.0.clone())
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|v| self.0.is_same_type_as(&v.0))
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        target
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|v| self.0.is_included_in(&v.0))
    }
    fn subtypes(&self, acc: &mut Type) {
        // The type of an empty list is a list where the items are `<unreachable>`
        acc.add(Arc::new(Self(Type::empty())));
        // All possible list types
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
    fn simplify_for_display(&self, info: &crate::program::run::CheckInfo) -> Option<Type> {
        Some(Type::new(Self(self.0.simplify_for_display(info))))
    }
}
impl List {
    pub fn inner_type(&self) -> Type {
        let mut t = Type::empty();
        for el in &self.0 {
            t.add_all(&el.get().as_type());
        }
        t
    }
}
