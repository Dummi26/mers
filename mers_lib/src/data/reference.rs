use std::{any::Any, fmt::Display, sync::Mutex};

use super::{Data, MersData, MersType, Type};

#[derive(Debug, Clone)]
pub struct Reference(pub Data);

impl MersData for Reference {
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
pub struct ReferenceT(pub Type);
impl MersType for ReferenceT {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.0.is_same_type_as(&o.0)
        } else {
            false
        }
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

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&{}", self.0.get())
    }
}
