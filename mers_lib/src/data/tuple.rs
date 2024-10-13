use std::{any::Any, sync::Arc};

use crate::{errors::CheckError, info::DisplayInfo};

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
    fn display(&self, info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            c.get().display(info, f)?;
        }
        write!(f, ")")?;
        Ok(())
    }
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.0 == self.0
        } else {
            false
        }
    }
    fn iterable(
        &self,
        _gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        Some(Box::new(self.0.clone().into_iter().map(Ok)))
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
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c.with_display(info))?;
        }
        write!(f, ")")?;
        Ok(())
    }
    fn iterable(&self) -> Option<Type> {
        let mut o = Type::empty();
        for t in self.0.iter() {
            o.add_all(&t);
        }
        Some(o)
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.0.len() == other.0.len()
                && self
                    .0
                    .iter()
                    .zip(other.0.iter())
                    .all(|(s, o)| s.is_same_type_as(o))
        } else {
            false
        }
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            self.0.len() == target.0.len()
                && self
                    .0
                    .iter()
                    .zip(target.0.iter())
                    .all(|(s, t)| s.is_included_in(t))
        } else {
            false
        }
    }
    fn subtypes(&self, acc: &mut Type) {
        self.gen_subtypes_recursively(acc, &mut Vec::with_capacity(self.0.len()));
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
    fn simplify_for_display(&self, info: &crate::program::run::CheckInfo) -> Option<Type> {
        Some(Type::new(Self(
            self.0
                .iter()
                .map(|v| v.simplify_for_display(info))
                .collect(),
        )))
    }
}

impl TupleT {
    pub fn gen_subtypes_recursively(&self, acc: &mut Type, types: &mut Vec<Arc<dyn MersType>>) {
        if types.len() >= self.0.len() {
            let nt = Self(
                types
                    .iter()
                    .map(|v| Type::newm(vec![Arc::clone(v)]))
                    .collect(),
            );
            acc.add(Arc::new(nt));
        } else {
            for t in self.0[types.len()].subtypes_type().types {
                types.push(t);
                self.gen_subtypes_recursively(acc, types);
                types.pop();
            }
        }
    }
}
