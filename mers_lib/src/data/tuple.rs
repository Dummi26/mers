use std::{any::Any, fmt::Display};

use super::{Data, MersData, MersType, Type};

#[derive(Debug, Clone)]
pub struct Tuple(pub Vec<Data>);

impl Tuple {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, i: usize) -> Option<&Data> {
        self.0.get(i)
    }
}

impl MersData for Tuple {
    fn matches(&self) -> Option<Data> {
        if let Some(d) = self.0.first() {
            if self.0.len() == 1 {
                Some(d.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        Some(Box::new(self.0.clone().into_iter()))
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn to_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

#[derive(Debug)]
pub struct TupleT(pub Vec<Type>);
impl MersType for TupleT {
    fn matches(&self) -> Option<(Type, bool)> {
        if let Some(d) = self.0.first() {
            if self.0.len() == 1 {
                Some((d.clone(), true))
            } else {
                None
            }
        } else {
            None
        }
    }
    fn iterable(&self) -> Option<Type> {
        Some(todo!("joine types"))
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other.as_any().downcast_ref::<Self>().is_some()
    }
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        self.is_same_type_as(target)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn to_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

impl Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c.get())?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
