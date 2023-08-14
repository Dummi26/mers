use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    data::{self, Data, MersData, MersType, Type},
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// Adds a simple list type
    /// `List` can store a variable number of items
    /// `as_list: fn` turns a tuple into a list
    pub fn with_list(self) -> Self {
        // TODO: Type with generics
        self.add_type("List".to_string(), Type::new(ListT(Type::empty_tuple())))
            .add_var(
                "as_list".to_string(),
                Data::new(data::function::Function {
                    info: Arc::new(program::run::Info::neverused()),
                    info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                    out: Arc::new(|a, i| {
                        if let Some(v) = a.iterable() {
                            Ok(Type::new(ListT(v)))
                        } else {
                            Err(program::run::CheckError(format!(
                                "cannot iterate over type {a}"
                            )))
                        }
                    }),
                    run: Arc::new(|a, _i| {
                        if let Some(i) = a.get().iterable() {
                            Data::new(List(i.collect()))
                        } else {
                            unreachable!("as_list called on non-iterable")
                        }
                    }),
                }),
            )
    }
}

#[derive(Clone, Debug)]
pub struct List(Vec<Data>);
#[derive(Debug)]
pub struct ListT(Type);
impl MersData for List {
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.0 == self.0
        } else {
            false
        }
    }
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        Some(Box::new(self.0.clone().into_iter()))
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        let mut t = Type::empty();
        for el in &self.0 {
            t.add(Arc::new(el.get().as_type()));
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
        self.is_same_type_as(target)
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
            write!(f, "{}", c.get())?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
impl Display for ListT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)?;
        Ok(())
    }
}
