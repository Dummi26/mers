use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::program::{self, run::Info};

use super::{Data, MersData, MersType, Type};

#[derive(Clone)]
pub struct Function {
    pub info: Info,
    pub out: Arc<dyn Fn(&Type) -> Option<Type>>,
    pub run: Arc<dyn Fn(Data, &mut crate::program::run::Info) -> Data>,
}
impl Function {
    pub fn with_info(&self, info: program::run::Info) -> Self {
        Self {
            info,
            out: Arc::clone(&self.out),
            run: Arc::clone(&self.run),
        }
    }
    pub fn run(&self, arg: Data) -> Data {
        (self.run)(arg, &mut self.info.clone())
    }
}

impl MersData for Function {
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

pub struct FunctionT(Arc<dyn Fn(&Type) -> Option<Type>>);
impl MersType for FunctionT {
    fn is_same_type_as(&self, _other: &dyn MersType) -> bool {
        false
    }
    fn is_included_in_single(&self, _target: &dyn MersType) -> bool {
        false
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

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function")
    }
}
impl Debug for FunctionT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionT")
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function>")
    }
}
