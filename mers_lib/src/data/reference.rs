use std::{
    any::Any,
    sync::{Arc, RwLock},
};

use crate::{errors::CheckError, info::DisplayInfo};

use super::{Data, MersData, MersType, Type};

#[derive(Debug, Clone)]
pub struct Reference(pub Arc<RwLock<Data>>);

impl MersData for Reference {
    fn display(&self, info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.read().unwrap().get().display(info, f)
    }
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
        let inner = self.0.read().unwrap();
        let inner = inner.get();
        if let Some(func) = inner
            .as_ref()
            .as_any()
            .downcast_ref::<crate::data::function::Function>()
        {
            Some(func.get_as_type())
        } else {
            None
        }
    }
    fn execute(
        &self,
        arg: Data,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Result<Data, CheckError>> {
        let mut inner = self.0.write().unwrap();
        let mut inner = inner.get_mut();
        if let Some(func) = inner
            .as_mut()
            .mut_any()
            .downcast_mut::<crate::data::function::Function>()
        {
            Some(func.run_mut(arg, gi.clone()))
        } else {
            None
        }
    }
    fn iterable(
        &self,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        let inner = Arc::clone(&self.0);
        let gi = gi.clone();
        Some(Box::new(std::iter::from_fn(move || {
            match inner
                .write()
                .unwrap()
                .get_mut()
                .mut_any()
                .downcast_mut::<crate::data::function::Function>()
                .unwrap()
                .run_mut(Data::empty_tuple(), gi.clone())
            {
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
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            *other.0.write().unwrap() == *self.0.write().unwrap()
        } else {
            false
        }
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        Type::new(ReferenceT(self.0.read().unwrap().get().as_type()))
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

#[derive(Debug, Clone)]
pub struct ReferenceT(pub Type);
impl MersType for ReferenceT {
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if self.0.types.len() > 1 {
            write!(f, "&{{{}}}", self.0.with_display(info))
        } else {
            write!(f, "&{}", self.0.with_display(info))
        }
    }
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
        let mut funcs: Vec<crate::data::function::FunctionT> = vec![];
        for func in self.0.types.iter() {
            funcs.push(Clone::clone(
                func.as_any()
                    .downcast_ref::<crate::data::function::FunctionT>()?,
            ));
        }
        Some(super::function::FunctionT(
            Ok(Arc::new(move |a, _| {
                let mut out = Type::empty();
                for func in funcs.iter() {
                    out.add_all(&func.o(a)?);
                }
                Ok(out)
            })),
            crate::info::Info::neverused(),
        ))
    }
    fn iterable(&self) -> Option<Type> {
        let mut out = Type::empty();
        for func in self.0.types.iter() {
            out.add_all(
                &func
                    .as_any()
                    .downcast_ref::<crate::data::function::FunctionT>()?
                    .iterable()?,
            );
        }
        if !out.types.is_empty() {
            Some(out)
        } else {
            None
        }
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.0.is_same_type_as(&o.0)
        } else {
            false
        }
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        // &int isn't included in &(int/float), otherwise we could assign a float to it
        self.is_same_type_as(target)
    }
    fn subtypes(&self, acc: &mut Type) {
        // // we don't call subtypes because (int/string) must stay that so we can assign either
        // // NOTE: this might not be right...?
        // acc.add(Arc::new(self.clone()));
        // FOR NOW (until we can put the compile-time type in ReferenceT), add all these types, too
        // TODO: Figure out how to fix
        // x := if true 1 else 0.5
        // &x.debug // prints &Int instead of &{Int/Float} at runtime :(
        for t in self.0.subtypes_type().types {
            acc.add(Arc::new(Self(Type::newm(vec![t]))));
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
    fn is_reference_to(&self) -> Option<&Type> {
        Some(&self.0)
    }
}
