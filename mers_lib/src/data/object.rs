use std::{fmt::Display, sync::Arc};

use super::{Data, MersData, MersType, Type};

#[derive(Debug, PartialEq, Clone)]
pub struct Object(pub Vec<(String, Data)>);
#[derive(Debug, Clone)]
pub struct ObjectT(pub Vec<(String, Type)>);

impl MersData for Object {
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        Type::new(ObjectT(
            self.0
                .iter()
                .map(|(n, v)| (n.clone(), v.get().as_type()))
                .collect(),
        ))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
}

impl MersType for ObjectT {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other.as_any().downcast_ref::<Self>().is_some_and(|other| {
            self.0.len() == other.0.len()
                && self
                    .0
                    .iter()
                    .zip(other.0.iter())
                    .all(|((s1, t1), (s2, t2))| s1 == s2 && t1.is_same_type_as(t2))
        })
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        target
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|target| {
                self.0.len() >= target.0.len()
                    && self
                        .0
                        .iter()
                        .zip(target.0.iter())
                        .all(|((s1, t1), (s2, t2))| s1 == s2 && t1.is_included_in(t2))
            })
    }
    fn subtypes(&self, acc: &mut Type) {
        self.gen_subtypes_recursively(acc, &mut Vec::with_capacity(self.0.len()));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }
    fn simplify_for_display(&self, info: &crate::program::run::CheckInfo) -> Option<Type> {
        Some(Type::new(Self(
            self.0
                .iter()
                .map(|(n, t)| (n.clone(), t.simplify_for_display(info)))
                .collect(),
        )))
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut comma_sep = false;
        write!(f, "{{")?;
        for (name, val) in self.0.iter() {
            if comma_sep {
                write!(f, ", ")?;
            }
            write!(f, "{name}: {}", val.get())?;
            comma_sep = true;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
impl Display for ObjectT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut comma_sep = false;
        write!(f, "{{")?;
        for (name, t) in self.0.iter() {
            if comma_sep {
                write!(f, ", ")?;
            }
            write!(f, "{name}: {t}")?;
            comma_sep = true;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl ObjectT {
    pub fn gen_subtypes_recursively(
        &self,
        acc: &mut Type,
        types: &mut Vec<(String, Arc<dyn MersType>)>,
    ) {
        if types.len() >= self.0.len() {
            let nt = Self(
                types
                    .iter()
                    .map(|(s, v)| (s.clone(), Type::newm(vec![Arc::clone(v)])))
                    .collect(),
            );
            acc.add(Arc::new(nt));
        } else {
            for t in self.0[types.len()].1.subtypes_type().types {
                types.push((self.0[types.len()].0.clone(), t));
                self.gen_subtypes_recursively(acc, types);
                types.pop();
            }
        }
    }
}
