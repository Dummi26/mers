use std::{any::Any, fmt::Display};

use super::{MersData, MersType, Type};

#[derive(Debug, Clone)]
pub struct String(pub std::string::String);

impl MersData for String {
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
pub struct StringT;
impl MersType for StringT {
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

impl Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
