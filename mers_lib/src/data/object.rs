use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::info::DisplayInfo;

use super::{Data, MersData, MersDataWInfo, MersType, Type};

#[derive(Debug, PartialEq, Clone)]
pub struct Object(Vec<(usize, Data)>);
impl Object {
    pub fn new(v: Vec<(usize, Data)>) -> Self {
        Self(v)
    }
    pub fn get(&self, f: usize) -> Option<&Data> {
        self.iter().find(|v| v.0 == f).map(|v| &v.1)
    }
    pub fn iter(&self) -> std::slice::Iter<(usize, Data)> {
        self.0.iter()
    }
}
#[derive(Debug, Clone)]
pub struct ObjectT(Vec<(usize, Type)>);
impl ObjectT {
    pub fn new(v: Vec<(usize, Type)>) -> Self {
        Self(v)
    }
    pub fn get(&self, f: usize) -> Option<&Type> {
        self.iter().find(|v| v.0 == f).map(|v| &v.1)
    }
    pub fn iter(&self) -> std::slice::Iter<(usize, Type)> {
        self.0.iter()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl MersData for Object {
    fn display(&self, info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut comma_sep = false;
        write!(f, "{{")?;
        for (field, val) in self.iter() {
            if comma_sep {
                write!(f, ", ")?;
            }
            write!(
                f,
                "{}: {}",
                info.get_object_field_name(*field),
                val.get().with_display(info)
            )?;
            comma_sep = true;
        }
        write!(f, "}}")?;
        Ok(())
    }
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
            self.iter()
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
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        let mut comma_sep = false;
        write!(f, "{{")?;
        for (field, t) in self.iter() {
            if comma_sep {
                write!(f, ", ")?;
            }
            write!(
                f,
                "{}: {}",
                info.get_object_field_name(*field),
                t.with_display(info)
            )?;
            comma_sep = true;
        }
        write!(f, "}}")?;
        Ok(())
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        other.as_any().downcast_ref::<Self>().is_some_and(|other| {
            self.len() == other.len()
                && other.iter().all(|(field, target_type)| {
                    self.get(*field)
                        .is_some_and(|self_type| self_type.is_same_type_as(target_type))
                })
        })
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        target
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|target| {
                self.len() >= target.len()
                    && target.iter().all(|(field, target_type)| {
                        self.get(*field)
                            .is_some_and(|self_type| self_type.is_included_in(target_type))
                    })
            })
    }
    fn subtypes(&self, acc: &mut Type) {
        self.gen_subtypes_recursively(acc, &mut Vec::with_capacity(self.len()));
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

impl ObjectT {
    pub fn gen_subtypes_recursively(
        &self,
        acc: &mut Type,
        types: &mut Vec<(usize, Arc<dyn MersType>)>,
    ) {
        if types.len() >= self.len() {
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

pub trait ObjectFieldsMap {
    fn get_or_add_field(&self, field: &str) -> usize;
}
impl ObjectFieldsMap for Arc<Mutex<HashMap<String, usize>>> {
    fn get_or_add_field(&self, field: &str) -> usize {
        let mut s = self.lock().unwrap();
        if let Some(f) = s.get(field) {
            return *f;
        }
        let o = s.len();
        s.insert(field.to_owned(), o);
        o
    }
}
