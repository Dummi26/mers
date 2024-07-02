use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{
    errors::CheckError,
    info::Local,
    program::run::{CheckInfo, Info},
};

use super::{Data, MersData, MersType, Type};

pub struct Function {
    pub info: Info,
    pub info_check: Arc<Mutex<CheckInfo>>,
    pub out: Result<
        Arc<dyn Fn(&Type, &mut CheckInfo) -> Result<Type, CheckError> + Send + Sync>,
        Arc<Vec<(Type, Type)>>,
    >,
    pub run:
        Arc<dyn Fn(Data, &mut crate::program::run::Info) -> Result<Data, CheckError> + Send + Sync>,
    pub inner_statements: Option<(
        Arc<Box<dyn crate::program::run::MersStatement>>,
        Arc<Box<dyn crate::program::run::MersStatement>>,
    )>,
}
impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            info: self.info.duplicate(),
            info_check: self.info_check.clone(),
            out: self.out.clone(),
            run: self.run.clone(),
            inner_statements: self.inner_statements.clone(),
        }
    }
}
impl Function {
    pub fn new_static(
        out: Vec<(Type, Type)>,
        run: impl Fn(Data) -> Result<Data, CheckError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            info: crate::info::Info::neverused(),
            info_check: Arc::new(Mutex::new(crate::info::Info::neverused())),
            out: Err(Arc::new(out)),
            run: Arc::new(move |a, _| run(a)),
            inner_statements: None,
        }
    }
    pub fn new_generic(
        out: impl Fn(&Type) -> Result<Type, CheckError> + Send + Sync + 'static,
        run: impl Fn(Data) -> Result<Data, CheckError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            info: crate::info::Info::neverused(),
            info_check: Arc::new(Mutex::new(crate::info::Info::neverused())),
            out: Ok(Arc::new(move |a, _| out(a))),
            run: Arc::new(move |a, _| run(a)),
            inner_statements: None,
        }
    }
    pub fn with_info_run(&self, info: Info) -> Self {
        Self {
            info,
            info_check: Arc::clone(&self.info_check),
            out: self.out.clone(),
            run: Arc::clone(&self.run),
            inner_statements: self
                .inner_statements
                .as_ref()
                .map(|v| (Arc::clone(&v.0), Arc::clone(&v.1))),
        }
    }
    pub fn with_info_check(&self, check: CheckInfo) {
        *self.info_check.lock().unwrap() = check;
    }
    pub fn check(&self, arg: &Type) -> Result<Type, CheckError> {
        self.get_as_type().o(arg)
    }
    pub fn run_mut(&mut self, arg: Data) -> Result<Data, CheckError> {
        (self.run)(arg, &mut self.info)
    }
    pub fn run_immut(&self, arg: Data) -> Result<Data, CheckError> {
        (self.run)(arg, &mut self.info.duplicate())
    }
    pub fn get_as_type(&self) -> FunctionT {
        match &self.out {
            Ok(out) => {
                let out = Arc::clone(out);
                let info = self.info_check.lock().unwrap().clone();
                FunctionT(Ok(Arc::new(move |a| out(a, &mut info.clone()))))
            }
            Err(types) => FunctionT(Err(Arc::clone(types))),
        }
    }

    pub fn inner_statements(
        &self,
    ) -> &Option<(
        Arc<Box<dyn crate::program::run::MersStatement>>,
        Arc<Box<dyn crate::program::run::MersStatement>>,
    )> {
        &self.inner_statements
    }
}

impl MersData for Function {
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
        Some(self.get_as_type())
    }
    fn execute(&self, arg: Data) -> Option<Result<Data, CheckError>> {
        Some(self.run_immut(arg))
    }
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        let mut s = Clone::clone(self);
        Some(Box::new(std::iter::from_fn(move || {
            match s.run_mut(Data::empty_tuple()) {
                Err(e) => Some(Err(e)),
                Ok(v) => {
                    if let Some(v) = v.one_tuple_content() {
                        Some(Ok(v))
                    } else {
                        None
                    }
                }
            }
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
pub struct FunctionT(
    pub Result<Arc<dyn Fn(&Type) -> Result<Type, CheckError> + Send + Sync>, Arc<Vec<(Type, Type)>>>,
);
impl FunctionT {
    /// get output type
    pub fn o(&self, i: &Type) -> Result<Type, CheckError> {
        match &self.0 {
            Ok(f) => f(i),
            Err(v) => v.iter().find(|(a, _)| i.is_included_in(a)).map(|(_, o)| o.clone()).ok_or_else(|| format!("This function, which was defined with an explicit type, cannot be called with an argument of type {i}.").into()),
        }
    }
}
impl MersType for FunctionT {
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
        Some(self.clone())
    }
    fn iterable(&self) -> Option<Type> {
        // if this function can be called with an empty tuple and returns `()` or `(T)`, it can act as an iterator with type `T`.
        if let Ok(t) = self.o(&Type::empty_tuple()) {
            let mut out = Type::empty();
            for t in &t.types {
                if let Some(t) = t.as_any().downcast_ref::<super::tuple::TupleT>() {
                    if t.0.len() > 1 {
                        return None;
                    } else if let Some(t) = t.0.first() {
                        out.add_all(&t);
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
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Err(s) = &self.0 {
            if let Some(other) = other.as_any().downcast_ref::<Self>() {
                if let Err(o) = &other.0 {
                    s.iter().all(|(si, so)| {
                        o.iter()
                            .any(|(oi, oo)| si.is_same_type_as(oi) && so.is_same_type_as(oo))
                    }) && o.iter().all(|(oi, oo)| {
                        s.iter()
                            .any(|(si, so)| oi.is_same_type_as(si) && oo.is_same_type_as(so))
                    })
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            if let Err(s) = &target.0 {
                s.iter()
                    .all(|(i, o)| self.o(i).is_ok_and(|r| r.is_included_in(o)))
            } else {
                false
            }
        } else {
            false
        }
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
        match &self.0 {
            Err(e) => {
                write!(f, "(")?;
                for (index, (i, o)) in e.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{i} -> {o}")?;
                }
                write!(f, ")")
            }
            Ok(_) => match self.o(&Type::empty_tuple()) {
                Ok(t) => write!(f, "(() -> {t}, ...)"),
                Err(_) => {
                    write!(f, "(... -> ...)",)
                }
            },
        }
    }
}
