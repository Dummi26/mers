use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::program::run::{CheckError, CheckInfo, Info};

use super::{Data, MersData, MersType, Type};

#[derive(Clone)]
pub struct Function {
    pub info: Arc<Info>,
    pub info_check: Arc<Mutex<CheckInfo>>,
    pub out: Arc<dyn Fn(&Type, &mut CheckInfo) -> Result<Type, CheckError> + Send + Sync>,
    pub run: Arc<dyn Fn(Data, &mut crate::program::run::Info) -> Data + Send + Sync>,
}
impl Function {
    pub fn with_info_run(&self, info: Arc<Info>) -> Self {
        Self {
            info,
            info_check: Arc::clone(&self.info_check),
            out: Arc::clone(&self.out),
            run: Arc::clone(&self.run),
        }
    }
    pub fn with_info_check(&self, check: CheckInfo) {
        *self.info_check.lock().unwrap() = check;
    }
    pub fn check(&self, arg: &Type) -> Result<Type, CheckError> {
        (self.out)(arg, &mut self.info_check.lock().unwrap().clone())
    }
    pub fn run(&self, arg: Data) -> Data {
        (self.run)(arg, &mut self.info.as_ref().clone())
    }
}

impl MersData for Function {
    fn is_eq(&self, _other: &dyn MersData) -> bool {
        false
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        let out = Arc::clone(&self.out);
        let info = Arc::clone(&self.info_check);
        Type::new(FunctionT(Arc::new(move |a| {
            out(a, &mut info.lock().unwrap().clone())
        })))
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

pub struct FunctionT(pub Arc<dyn Fn(&Type) -> Result<Type, CheckError> + Send + Sync>);
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
impl Display for FunctionT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function")
    }
}
