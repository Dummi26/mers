use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub mod bool;
pub mod float;
pub mod function;
pub mod int;
pub mod reference;
pub mod string;
pub mod tuple;

pub mod defs;

pub trait MersData: Any + Debug + Display {
    fn matches(&self) -> Option<Data> {
        None
    }
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        None
    }
    /// By default, uses `iterable` to get an iterator and `nth` to retrieve the nth element.
    /// Should have a custom implementation for better performance on most types
    fn get(&self, i: usize) -> Option<Data> {
        self.iterable()?.nth(i)
    }
    fn clone(&self) -> Box<dyn MersData>;
    fn as_any(&self) -> &dyn Any;
    fn mut_any(&mut self) -> &mut dyn Any;
    fn to_any(self) -> Box<dyn Any>;
}

pub trait MersType: Any + Debug {
    /// If Some((_, false)) is returned, data of this type could match. If it matches, it matches with the type.
    /// If Some((_, true)) is returned, data of this type will always match with the type.
    fn matches(&self) -> Option<(Type, bool)> {
        None
    }
    /// If Some(T), calling `iterable` on the MersData this MersType belongs to
    /// Should return Some(I), where I is an Iterator which only returns items of type T.
    fn iterable(&self) -> Option<Type> {
        None
    }
    /// If Some(T), calling `get` on data of this type may return T, but it might also return None.
    /// By default, this returns the same thing as `iterable`, since this is also the default implementation for `MersData::get`.
    fn get(&self) -> Option<Type> {
        self.iterable()
    }
    /// If self and other are different types (`other.as_any().downcast_ref::<Self>().is_none()`),
    /// this *must* return false.
    fn is_same_type_as(&self, other: &dyn MersType) -> bool;
    /// This doesn't handle the case where target is Type (is_included_in handles it)
    fn is_included_in_single(&self, target: &dyn MersType) -> bool;
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Type>() {
            target
                .types
                .iter()
                .any(|t| self.is_included_in_single(t.as_ref()))
        } else {
            self.is_included_in_single(target)
        }
    }
    fn as_any(&self) -> &dyn Any;
    fn mut_any(&mut self) -> &mut dyn Any;
    fn to_any(self) -> Box<dyn Any>;
}

#[derive(Debug)]
pub struct Data {
    pub data: Arc<RwLock<Box<dyn MersData>>>,
}
impl Data {
    pub fn new<T: MersData>(data: T) -> Self {
        Self::new_boxed(Box::new(data))
    }
    pub fn new_boxed(data: Box<dyn MersData>) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }
    pub fn empty_tuple() -> Self {
        Self::new(tuple::Tuple(vec![]))
    }
    pub fn one_tuple(v: Self) -> Self {
        Self::new(tuple::Tuple(vec![v]))
    }
    pub fn get(&self) -> RwLockReadGuard<Box<dyn MersData>> {
        self.data.read().unwrap()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<Box<dyn MersData>> {
        self.data.write().unwrap()
    }
}
impl Clone for Data {
    fn clone(&self) -> Self {
        // todo!("clone for data - requires CoW");
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Type {
    // TODO: Maybe make sure this is always sorted by (recursive?!?) TypeId,
    // that way is_same_type_as can work more efficiently (cuz good code but also branch prediction)
    types: Vec<Arc<dyn MersType>>,
}
impl Type {
    pub fn new<T: MersType>(t: T) -> Self {
        Self {
            types: vec![Arc::new(t)],
        }
    }
    pub fn newm(types: Vec<Arc<dyn MersType>>) -> Self {
        Self { types }
    }
    pub fn empty_tuple() -> Self {
        Self::new(tuple::TupleT(vec![]))
    }
    pub fn add<T: MersType>(&mut self, new: Box<T>) {
        todo!()
    }
}

// PROBLEM:
// [int, int]/[int, string]/[string, int]/[string, string]
// is the same type as [int/string, int/string],
// but [int, int]/[int, string]/[string int] isn't.
// somehow, we need to merge these into the simplest form (same outer type, inner types differ)
// before we can implement `Type`
// idea: pick all the ones with the same first type: [int, int]/[int, string] and [string, int]/[string, string]
// then repeat with the second type if possible (here not, but for longer tuples, probably?)
// merge the last existing type in all the collections until we reach the first type again or the last types aren't equal anymore (how to check????)

impl MersType for Type {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        todo!()
    }
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        todo!()
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
