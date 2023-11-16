use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{
    errors::CheckError,
    program::run::{CheckInfo, Info},
};

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
        let lock = self.info_check.lock().unwrap();
        let mut info = lock.clone();
        drop(lock);
        (self.out)(arg, &mut info)
    }
    pub fn run(&self, arg: Data) -> Data {
        (self.run)(arg, &mut self.info.as_ref().clone())
    }
    pub fn get_as_type(&self) -> FunctionT {
        let out = Arc::clone(&self.out);
        let info = Arc::clone(&self.info_check);
        FunctionT(Arc::new(move |a| {
            let lock = info.lock().unwrap();
            let mut info = lock.clone();
            drop(lock);
            out(a, &mut info)
        }))
    }
}

impl MersData for Function {
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        let s = Clone::clone(self);
        Some(Box::new(std::iter::from_fn(move || {
            s.run(Data::empty_tuple()).one_tuple_content()
        })))
    }
    fn is_eq(&self, _other: &dyn MersData) -> bool {
        false
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        Type::new(self.get_as_type())
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

#[derive(Clone)]
pub struct FunctionT(pub Arc<dyn Fn(&Type) -> Result<Type, CheckError> + Send + Sync>);
impl MersType for FunctionT {
    fn iterable(&self) -> Option<Type> {
        // if this function can be called with an empty tuple and returns `()` or `(T)`, it can act as an iterator with type `T`.
        if let Ok(t) = self.0(&Type::empty_tuple()) {
            let mut out = Type::empty();
            for t in &t.types {
                if let Some(t) = t.as_any().downcast_ref::<super::tuple::TupleT>() {
                    if t.0.len() > 1 {
                        return None;
                    } else if let Some(t) = t.0.first() {
                        out.add(Arc::new(t.clone()))
                    }
                } else {
                    return None;
                }
            }
            Some(out)
        } else {
            None
        }
    }
    fn is_same_type_as(&self, _other: &dyn MersType) -> bool {
        false
    }
    fn is_included_in_single(&self, _target: &dyn MersType) -> bool {
        false
    }
    fn subtypes(&self, acc: &mut Type) {
        acc.add(Arc::new(self.clone()));
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
        match (self.0)(&Type::empty_tuple()) {
            Ok(t) => write!(f, "Function /* () -> {t} */"),
            Err(_) => {
                write!(f, "Function",)
            }
        }
    }
}
