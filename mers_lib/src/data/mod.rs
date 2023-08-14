use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{atomic::AtomicUsize, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub mod bool;
pub mod float;
pub mod function;
pub mod int;
pub mod reference;
pub mod string;
pub mod tuple;

pub mod defs;

pub trait MersData: Any + Debug + Display + Send + Sync {
    fn iterable(&self) -> Option<Box<dyn Iterator<Item = Data>>> {
        None
    }
    /// By default, uses `iterable` to get an iterator and `nth` to retrieve the nth element.
    /// Should have a custom implementation for better performance on most types
    fn get(&self, i: usize) -> Option<Data> {
        self.iterable()?.nth(i)
    }
    /// If self and other are different types (`other.as_any().downcast_ref::<Self>().is_none()`),
    /// this *must* return false.
    fn is_eq(&self, other: &dyn MersData) -> bool;
    fn clone(&self) -> Box<dyn MersData>;
    fn as_type(&self) -> Type;
    fn as_any(&self) -> &dyn Any;
    fn mut_any(&mut self) -> &mut dyn Any;
    fn to_any(self) -> Box<dyn Any>;
}

pub trait MersType: Any + Debug + Display + Send + Sync {
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
    fn is_reference_to(&self) -> Option<&Type> {
        None
    }
}

#[derive(Debug)]
pub struct Data {
    is_mut: bool,
    counts: Arc<(AtomicUsize, AtomicUsize)>,
    pub data: Arc<RwLock<Box<dyn MersData>>>,
}
impl Data {
    pub fn new<T: MersData>(data: T) -> Self {
        Self::new_boxed(Box::new(data), true)
    }
    pub fn new_boxed(data: Box<dyn MersData>, is_mut: bool) -> Self {
        Self {
            is_mut,
            counts: Arc::new((
                AtomicUsize::new(if is_mut { 0 } else { 1 }),
                AtomicUsize::new(if is_mut { 1 } else { 0 }),
            )),
            data: Arc::new(RwLock::new(data)),
        }
    }
    pub fn empty_tuple() -> Self {
        Self::new(tuple::Tuple(vec![]))
    }
    pub fn one_tuple(v: Self) -> Self {
        Self::new(tuple::Tuple(vec![v]))
    }
    /// Returns true if self is `()`.
    pub fn is_zero_tuple(&self) -> bool {
        if let Some(tuple) = self
            .get()
            .as_any()
            .downcast_ref::<crate::data::tuple::Tuple>()
        {
            tuple.0.is_empty()
        } else {
            false
        }
    }
    /// Returns `Some(d)` if and only if self is `(d)`.
    pub fn one_tuple_content(&self) -> Option<Data> {
        if let Some(data) = self
            .get()
            .as_any()
            .downcast_ref::<crate::data::tuple::Tuple>()
            .filter(|v| v.len() == 1)
            .and_then(|v| v.get(0))
        {
            Some(data.clone())
        } else {
            None
        }
    }
    pub fn get(&self) -> RwLockReadGuard<Box<dyn MersData>> {
        #[cfg(debug_assertions)]
        eprintln!("[mers:data:cow] get");
        self.data.read().unwrap()
    }
    pub fn get_mut_unchecked(&self) -> RwLockWriteGuard<Box<dyn MersData>> {
        self.data.write().unwrap()
    }
    pub fn try_get_mut(&self) -> Option<RwLockWriteGuard<Box<dyn MersData>>> {
        if self.is_mut && self.counts.0.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            Some(self.get_mut_unchecked())
        } else {
            None
        }
    }
    /// like try_get_mut, but instead of returning `None` this function `get()`s the data and clones it.
    /// When cloning data, this transforms `self` into a `Data` with `is_mut: true`, hence the `&mut self` parameter.
    pub fn get_mut(&mut self) -> RwLockWriteGuard<Box<dyn MersData>> {
        if self.try_get_mut().is_none() {
            #[cfg(debug_assertions)]
            eprintln!(
                "[mers:data:cow] cloning! get_mut called on {}",
                if !self.is_mut {
                    "non-mut value"
                } else {
                    "value with immut references"
                }
            );
            let val = self.get().clone();
            *self = Self::new_boxed(val, true);
        }
        self.get_mut_unchecked()
    }
    pub fn mkref(&self) -> Self {
        if self.is_mut {
            self.counts
                .1
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Self {
                is_mut: true,
                counts: Arc::clone(&self.counts),
                data: Arc::clone(&self.data),
            }
        } else {
            #[cfg(debug_assertions)]
            eprintln!("[mers:data:cow] cloning! mkref called on immutable data");
            Self::new_boxed(self.data.read().unwrap().clone(), true)
        }
    }
}
impl Clone for Data {
    fn clone(&self) -> Self {
        self.counts
            .0
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            is_mut: false,
            counts: Arc::clone(&self.counts),
            data: Arc::clone(&self.data),
        }
    }
}
impl Drop for Data {
    fn drop(&mut self) {
        if self.is_mut {
            &self.counts.1
        } else {
            &self.counts.0
        }
        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.get().is_eq(other.get().as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct Type {
    // TODO: Maybe make sure this is always sorted by (recursive?!?) TypeId,
    // that way is_same_type_as can work more efficiently (cuz good code but also branch prediction)
    pub types: Vec<Arc<dyn MersType>>,
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
    pub fn empty() -> Self {
        Self { types: vec![] }
    }
    pub fn empty_tuple() -> Self {
        Self::new(tuple::TupleT(vec![]))
    }
    /// Returns true if self is `()`.
    pub fn is_zero_tuple(&self) -> bool {
        let mut o = false;
        for t in &self.types {
            o = true;
            if let Some(tuple) = t.as_any().downcast_ref::<crate::data::tuple::TupleT>() {
                if !tuple.0.is_empty() {
                    return false;
                }
            } else {
                return false;
            }
        }
        o
    }
    /// Returns `Some(d)` if and only if self is `(d)`.
    pub fn one_tuple_content(&self) -> Option<Type> {
        let mut o = Self::empty();
        for t in &self.types {
            if let Some(t) = t
                .as_any()
                .downcast_ref::<crate::data::tuple::TupleT>()
                .filter(|v| v.0.len() == 1)
                .and_then(|v| v.0.get(0))
            {
                o.add(Arc::new(t.clone()));
            } else {
                return None;
            }
        }
        Some(o)
    }
    pub fn add(&mut self, new: Arc<dyn MersType>) {
        let n = new.as_any();
        if let Some(s) = n.downcast_ref::<Self>() {
            for t in &s.types {
                self.add(Arc::clone(t));
            }
        } else {
            self.types.push(new);
        }
    }
    pub fn dereference(&self) -> Option<Self> {
        let mut o = Self::empty();
        for t in &self.types {
            if let Some(t) = t.is_reference_to() {
                o.add(Arc::new(t.clone()));
            } else {
                return None;
            }
        }
        Some(o)
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
        self.types.iter().all(|t| t.is_included_in_single(target))
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
    fn iterable(&self) -> Option<Type> {
        let mut o = Self::empty();
        for t in self.types.iter() {
            if let Some(t) = t.iterable() {
                o.add(Arc::new(t));
            } else {
                return None;
            }
        }
        Some(o)
    }
    fn get(&self) -> Option<Type> {
        let mut o = Self::empty();
        for t in self.types.iter() {
            if let Some(t) = t.get() {
                o.add(Arc::new(t));
            } else {
                return None;
            }
        }
        Some(o)
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.types.is_empty() {
            write!(f, "<unreachable>")
        } else {
            if self.types.len() > 1 {
                write!(f, "{{")?;
            }
            write!(f, "{}", self.types[0])?;
            for t in self.types.iter().skip(1) {
                write!(f, "/{t}")?;
            }
            if self.types.len() > 1 {
                write!(f, "}}")?;
            }
            Ok(())
        }
    }
}
