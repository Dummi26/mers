use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use crate::{
    errors::CheckError,
    info::DisplayInfo,
    parsing::types::ParsedType,
    program::run::{CheckInfo, Info},
};

use super::{Data, MersData, MersType, Type};

pub struct Function {
    pub info: Info,
    pub info_check: Arc<Mutex<CheckInfo>>,
    pub fixed_type: Option<Vec<(Vec<ParsedType>, Option<Vec<ParsedType>>)>>,
    pub fixed_type_out: Arc<Mutex<Option<Result<Arc<Vec<(Type, Type)>>, CheckError>>>>,
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
            fixed_type: self.fixed_type.clone(),
            fixed_type_out: self.fixed_type_out.clone(),
            out: self.out.clone(),
            run: self.run.clone(),
            inner_statements: self.inner_statements.clone(),
        }
    }
}
impl Function {
    pub fn new_static(
        out: Vec<(Type, Type)>,
        run: impl Fn(Data, &mut Info) -> Result<Data, CheckError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            info: crate::info::Info::neverused(),
            info_check: Arc::new(Mutex::new(crate::info::Info::neverused())),
            fixed_type: None,
            fixed_type_out: Arc::new(Mutex::new(None)),
            out: Err(Arc::new(out)),
            run: Arc::new(run),
            inner_statements: None,
        }
    }
    pub fn new_generic(
        out: impl Fn(&Type, &mut CheckInfo) -> Result<Type, CheckError> + Send + Sync + 'static,
        run: impl Fn(Data, &mut Info) -> Result<Data, CheckError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            info: crate::info::Info::neverused(),
            info_check: Arc::new(Mutex::new(crate::info::Info::neverused())),
            fixed_type: None,
            fixed_type_out: Arc::new(Mutex::new(None)),
            out: Ok(Arc::new(move |a, i| out(a, i))),
            run: Arc::new(run),
            inner_statements: None,
        }
    }
    pub fn with_info_run(&self, info: Info) -> Self {
        Self {
            info,
            info_check: Arc::clone(&self.info_check),
            fixed_type: self.fixed_type.clone(),
            fixed_type_out: self.fixed_type_out.clone(),
            out: self.out.clone(),
            run: Arc::clone(&self.run),
            inner_statements: self
                .inner_statements
                .as_ref()
                .map(|v| (Arc::clone(&v.0), Arc::clone(&v.1))),
        }
    }
    pub fn with_info_check(&self, check: CheckInfo) {
        if let Some(fixed_type) = &self.fixed_type {
            if let Ok(out_func) = &self.out {
                let mut nout = Ok(Vec::with_capacity(fixed_type.len()));
                for (in_type, out_type) in fixed_type {
                    if let Ok(new_out) = &mut nout {
                        let in_type = crate::parsing::types::type_from_parsed(in_type, &check)
                            .expect("failed to get intype from parsed type");
                        let out_type_want = out_type.as_ref().map(|out_type| {
                            crate::parsing::types::type_from_parsed(out_type, &check)
                                .expect("failed to get intype from parsed type")
                        });
                        let mut out_type = match out_func(&in_type, &mut check.clone()) {
                            Ok(t) => t,
                            Err(e) => {
                                nout = Err(e);
                                break;
                            }
                        };
                        if let Some(out_type_want) = out_type_want {
                            if out_type.is_included_in(&out_type_want) {
                                out_type = out_type_want;
                            } else {
                                nout = Err(format!(
                                        "function must return {} for input {} because of its definition, but it returns {}.",
                                        out_type_want.with_info(&check),
                                        in_type.with_info(&check),
                                        out_type.with_info(&check),
                                    )
                                    .into());
                                break;
                            }
                        }
                        new_out.push((in_type, out_type));
                    } else {
                        break;
                    }
                }
                *self.fixed_type_out.lock().unwrap() = Some(nout.map(Arc::new));
            }
        }
        *self.info_check.lock().unwrap() = check;
    }
    pub fn check(&self, arg: &Type) -> Result<Type, CheckError> {
        // TODO: this should require a CheckInfo and call `with_info_check`.
        self.get_as_type().o(arg)
    }
    pub fn check_try(&self, arg: &Type) -> Result<Type, (CheckError, Vec<(Type, Type)>)> {
        // TODO: this should require a CheckInfo and call `with_info_check`.
        self.get_as_type().o_try(arg)
    }
    pub fn run_mut(
        &mut self,
        arg: Data,
        gi: crate::program::run::RunLocalGlobalInfo,
    ) -> Result<Data, CheckError> {
        self.info.global = gi;
        self.run_mut_with_prev_gi(arg)
    }
    pub fn run_mut_with_prev_gi(&mut self, arg: Data) -> Result<Data, CheckError> {
        (self.run)(arg, &mut self.info)
    }
    pub fn run_immut(
        &self,
        arg: Data,
        gi: crate::program::run::RunLocalGlobalInfo,
    ) -> Result<Data, CheckError> {
        let mut i = self.info.duplicate();
        i.global = gi;
        (self.run)(arg, &mut i)
    }
    pub fn get_as_type(&self) -> FunctionT {
        let info = self.info_check.lock().unwrap().clone();
        if self.fixed_type.is_some() {
            match &*self.fixed_type_out.lock().unwrap() {
                Some(Ok(types)) => return FunctionT(Err(types.clone()), info),
                Some(Err(e)) => {
                    let e = e.clone();
                    return FunctionT(
                        Ok(Arc::new(move |_, _| Err(e.clone()))),
                        crate::info::Info::neverused(),
                    );
                }
                _ => {}
            }
        }
        match &self.out {
            Ok(out) => {
                let out = Arc::clone(out);
                FunctionT(Ok(Arc::new(move |a, i| out(a, &mut i.clone()))), info)
            }
            Err(types) => FunctionT(Err(Arc::clone(types)), info),
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
    fn display(&self, _info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
        Some(self.get_as_type())
    }
    fn execute(
        &self,
        arg: Data,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Result<Data, CheckError>> {
        Some(self.run_immut(arg, gi.clone()))
    }
    fn iterable(
        &self,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        let mut s = Clone::clone(self);
        let mut gi = Some(gi.clone());
        Some(Box::new(std::iter::from_fn(move || {
            match if let Some(gi) = gi.take() {
                s.run_mut(Data::empty_tuple(), gi)
            } else {
                s.run_mut_with_prev_gi(Data::empty_tuple())
            } {
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
    pub  Result<
        Arc<dyn Fn(&Type, &CheckInfo) -> Result<Type, CheckError> + Send + Sync>,
        Arc<Vec<(Type, Type)>>,
    >,
    pub CheckInfo,
);
impl FunctionT {
    /// get output type
    pub fn o(&self, i: &Type) -> Result<Type, CheckError> {
        self.o_try(i).map_err(|(e, _)| e)
    }
    /// get output type
    pub fn o_try(&self, i: &Type) -> Result<Type, (CheckError, Vec<(Type, Type)>)> {
        match &self.0 {
            Ok(f) => f(i, &self.1).map_err(|e| (e, vec![])),
            Err(v) => v
                .iter()
                .find(|(a, _)| i.is_included_in(a))
                .map(|(_, o)| o.clone())
                .ok_or_else(||
                    (
                        format!("This function, which was defined with an explicit type, cannot be called with an argument of type {}.", i.with_info(&self.1)).into(),
                        v.iter()
                            .filter(|(a, _)| i.types.iter().any(|i| a.types.iter().any(|a|
                                i.without(a.as_ref()).is_some())))
                            .map(|(a, o)| (a.clone(), o.clone()))
                            .collect()
                    )
                ),
        }
    }
}
impl MersType for FunctionT {
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match &self.0 {
            Err(e) => {
                write!(f, "(")?;
                for (index, (i, o)) in e.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} -> {}", i.with_display(info), o.with_display(info))?;
                }
                write!(f, ")")
            }
            Ok(_) => match self.o(&Type::empty_tuple()) {
                Ok(t) => write!(f, "(() -> {}, ...)", t.with_display(info)),
                Err(_) => {
                    write!(f, "(... -> ...)",)
                }
            },
        }
    }
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
    fn without(&self, remove: &dyn MersType) -> Option<Type> {
        if self.is_included_in(remove) {
            Some(Type::empty())
        } else {
            None
        }
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
