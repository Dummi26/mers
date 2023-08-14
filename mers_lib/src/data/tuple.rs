use std::{any::Any, fmt::Display, sync::Arc};

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
        Type::new(TupleT(self.0.iter().map(|v| v.get().as_type()).collect()))
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
    fn iterable(&self) -> Option<Type> {
        let mut o = Type::empty();
        for t in self.0.iter() {
            o.add(Arc::new(t.clone()));
        }
        Some(o)
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
impl Display for TupleT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
