use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{atomic::AtomicUsize, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{errors::CheckError, info::DisplayInfo};

pub mod bool;
pub mod byte;
pub mod float;
pub mod function;
pub mod int;
pub mod object;
pub mod reference;
pub mod string;
pub mod tuple;

pub mod defs;

pub trait MersData: Any + Debug + Send + Sync {
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result;
    /// must be the same as the `executable` impl on the MersType
    #[allow(unused_variables)]
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
        None
    }
    #[allow(unused_variables)]
    fn execute(
        &self,
        arg: Data,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Result<Data, CheckError>> {
        None
    }
    #[allow(unused_variables)]
    fn iterable(
        &self,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Box<dyn Iterator<Item = Result<Data, CheckError>>>> {
        None
    }
    /// By default, uses `iterable` to get an iterator and `nth` to retrieve the nth element.
    /// Should have a custom implementation for better performance on most types
    fn get(
        &self,
        i: usize,
        gi: &crate::program::run::RunLocalGlobalInfo,
    ) -> Option<Result<Data, CheckError>> {
        self.iterable(gi)?.nth(i)
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

pub trait MersDataWInfo {
    fn with_display<'a>(&'a self, info: &DisplayInfo<'a>) -> MersDataWithInfo<'a, Self> {
        MersDataWithInfo(self, *info)
    }
    fn with_info<'a>(
        &'a self,
        info: &'a crate::info::Info<impl crate::info::Local>,
    ) -> MersDataWithInfo<'a, Self> {
        MersDataWithInfo(self, info.display_info())
    }
}
pub trait MersTypeWInfo {
    fn with_display<'a>(&'a self, info: &DisplayInfo<'a>) -> MersTypeWithInfo<'a, Self> {
        MersTypeWithInfo(self, *info)
    }
    fn with_info<'a>(
        &'a self,
        info: &'a crate::info::Info<impl crate::info::Local>,
    ) -> MersTypeWithInfo<'a, Self> {
        MersTypeWithInfo(self, info.display_info())
    }
}
impl Type {
    pub fn with_display<'a>(&'a self, info: &DisplayInfo<'a>) -> TypeWithInfo<'a> {
        TypeWithInfo(self, *info)
    }
    pub fn with_info<'a>(
        &'a self,
        info: &'a crate::info::Info<impl crate::info::Local>,
    ) -> TypeWithInfo<'a> {
        TypeWithInfo(self, info.display_info())
    }
}
impl<T: MersData + ?Sized> MersDataWInfo for T {}
impl<T: MersType + ?Sized> MersTypeWInfo for T {}
pub struct MersDataWithInfo<'a, T: ?Sized>(&'a T, crate::info::DisplayInfo<'a>);
pub struct MersTypeWithInfo<'a, T: ?Sized>(&'a T, crate::info::DisplayInfo<'a>);
pub struct TypeWithInfo<'a>(&'a Type, crate::info::DisplayInfo<'a>);
impl<'a, T: ?Sized + MersData> Display for MersDataWithInfo<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display(&self.1, f)
    }
}
impl<'a, T: ?Sized + MersType> Display for MersTypeWithInfo<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display(&self.1, f)
    }
}
impl<'a> Display for TypeWithInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display(&self.1, f)
    }
}

pub trait MersType: Any + Debug + Send + Sync {
    fn display(
        &self,
        info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result;
    #[allow(unused_variables)]
    fn executable(&self) -> Option<crate::data::function::FunctionT> {
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
    /// This doesn't handle the case where target is Type (Type::is_included_in handles it)
    fn is_included_in(&self, target: &dyn MersType) -> bool;
    /// Returns all types that can result from the use of this type.
    /// Usually, this is just `acc.add(Arc::new(self.clone()))`
    /// but if there exists one or more inner types, this becomes interesting:
    /// Using `(int/string)` will end up being either `(int)` or `(string)`,
    /// so this function should add `(int)` and `(string)`.
    /// Since `(int/string)` can't exist at runtime, we don't need to list `self`.
    /// note also: `subtypes` has to be called recursively, i.e. you would have to call `.substring` on `int` and `string`.
    fn subtypes(&self, acc: &mut Type);
    /// like `subtypes`, but returns the accumulator
    fn subtypes_type(&self) -> Type {
        let mut acc = Type::empty();
        self.subtypes(&mut acc);
        acc
    }
    fn as_any(&self) -> &dyn Any;
    fn mut_any(&mut self) -> &mut dyn Any;
    fn to_any(self) -> Box<dyn Any>;
    fn is_reference_to(&self) -> Option<&Type> {
        None
    }
    /// may mutate `self` to simplify it
    #[allow(unused)]
    fn simplify_for_display(&self, info: &crate::program::run::CheckInfo) -> Option<Type> {
        None
    }
    fn simplified_as_string(&self, info: &crate::program::run::CheckInfo) -> String {
        self.simplify_for_display(info)
            .map(|s| s.with_info(info).to_string())
            .unwrap_or_else(|| self.with_info(info).to_string())
    }
}
#[derive(Clone, Debug)]
pub(crate) struct TypeWithOnlyName(pub(crate) String);
impl MersType for TypeWithOnlyName {
    fn display(
        &self,
        _info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "{self}")
    }
    fn is_same_type_as(&self, _other: &dyn MersType) -> bool {
        false
    }
    fn is_included_in(&self, _target: &dyn MersType) -> bool {
        false
    }
    fn subtypes(&self, acc: &mut Type) {
        acc.add(Arc::new(self.clone()))
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
impl Display for TypeWithOnlyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
    pub fn is_mut(&self) -> bool {
        self.is_mut
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
    /// If self is already mut, returns `(Data, false)`. If not, inner data will be cloned and `(Data, true)` will be returned.
    pub fn mkref(&self) -> (Self, bool) {
        if self.is_mut {
            self.counts
                .1
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            (
                Self {
                    is_mut: true,
                    counts: Arc::clone(&self.counts),
                    data: Arc::clone(&self.data),
                },
                false,
            )
        } else {
            #[cfg(debug_assertions)]
            eprintln!("[mers:data:cow] cloning! mkref called on immutable data");
            (
                Self::new_boxed(self.data.read().unwrap().clone(), true),
                true,
            )
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
    pub smart_type_simplification: bool,
}
impl Type {
    pub fn new<T: MersType>(t: T) -> Self {
        Self {
            types: vec![Arc::new(t)],
            smart_type_simplification: true,
        }
    }
    pub fn newm(types: Vec<Arc<dyn MersType>>) -> Self {
        Self {
            types,
            smart_type_simplification: true,
        }
    }
    pub fn empty() -> Self {
        Self {
            types: vec![],
            smart_type_simplification: true,
        }
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
                o.add_all(&t);
            } else {
                return None;
            }
        }
        Some(o)
    }
    /// Returns `Some(d)` if self is `()/(d)`
    pub fn one_tuple_possible_content(&self) -> Option<Type> {
        let mut o = Self::empty();
        let mut nothing = true;
        for t in &self.types {
            if let Some(t) = t
                .as_any()
                .downcast_ref::<crate::data::tuple::TupleT>()
                .filter(|v| v.0.len() == 1)
                .and_then(|v| v.0.get(0))
            {
                nothing = false;
                o.add_all(&t);
            }
        }
        if nothing {
            None
        } else {
            Some(o)
        }
    }
    pub fn add(&mut self, new: Arc<dyn MersType>) {
        let n = new.as_any();
        if let Some(s) = n.downcast_ref::<Self>() {
            self.add_all(s);
        } else if let Some(n) = self
            .smart_type_simplification
            .then(|| n.downcast_ref::<crate::data::int::IntT>())
            .flatten()
        {
            let n = n.clone();
            let mut newt = None;
            for a in &self.types {
                if let Some(t) = a.as_any().downcast_ref::<crate::data::int::IntT>() {
                    if t.0 <= n.0 && n.1 <= t.1 {
                        // we are included in this type
                        return;
                    }
                    if t.0 <= n.1.saturating_add(1) && n.0.saturating_sub(1) <= t.1 {
                        // this type will be added instead of the original `new`, and `t` will be removed from `self.types`.
                        newt = Some(crate::data::int::IntT(t.0.min(n.0), t.1.max(n.1)));
                        break;
                    }
                }
            }
            // remove types that are included in `self` before adding `self`
            let newt = newt.unwrap_or(n);
            let mut rmstack = vec![];
            for (i, a) in self.types.iter().enumerate() {
                if let Some(t) = a.as_any().downcast_ref::<crate::data::int::IntT>() {
                    if newt.0 <= t.0 && t.1 <= newt.1 {
                        rmstack.push(i);
                    }
                }
            }
            for i in rmstack.into_iter().rev() {
                self.types.remove(i);
            }
            self.types.push(Arc::new(newt));
        } else {
            if !self.types.iter().any(|t| new.is_included_in(t.as_ref())) {
                self.types.push(new);
            }
        }
    }
    pub fn add_all(&mut self, types: &Self) {
        for t in &types.types {
            self.add(Arc::clone(t));
        }
    }
    pub fn dereference(&self) -> Option<Self> {
        let mut o = Self::empty();
        for t in &self.types {
            if let Some(t) = t.is_reference_to() {
                o.add_all(&t);
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

impl Type {
    pub fn is_same_type_as(&self, other: &Self) -> bool {
        // TODO! improve
        self.is_included_in(other) && other.is_included_in(self)
    }
    pub fn is_included_in(&self, target: &Self) -> bool {
        self.types
            .iter()
            .all(|s| target.types.iter().any(|t| s.is_included_in(&**t)))
    }
    pub fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        self.types.iter().all(|s| s.is_included_in(target))
    }
    pub fn subtypes(&self, acc: &mut Type) {
        for t in &self.types {
            t.subtypes(acc);
        }
    }
    pub fn subtypes_type(&self) -> Type {
        let mut acc = Type::empty();
        acc.smart_type_simplification = false;
        self.subtypes(&mut acc);
        acc
    }
    pub fn iterable(&self) -> Option<Type> {
        let mut o = Self::empty();
        for t in self.types.iter() {
            o.add_all(&t.iterable()?);
        }
        Some(o)
    }
    pub fn get(&self) -> Option<Type> {
        let mut o = Self::empty();
        for t in self.types.iter() {
            if let Some(t) = t.get() {
                o.add_all(&t);
            } else {
                return None;
            }
        }
        Some(o)
    }
    pub fn simplify_for_display(&self, info: &crate::program::run::CheckInfo) -> Type {
        let mut out = Type::empty();
        'foreachtype: for ty in &self.types {
            // find the outmost type alias that isn't shadowed
            for (i, scope) in info.scopes.iter().enumerate() {
                if let Some((n, _)) = scope.types.iter().find(|(_, t)| {
                    t.as_ref()
                        .is_ok_and(|t| t.is_same_type_as(&Type::newm(vec![Arc::clone(ty)])))
                }) {
                    if info
                        .scopes
                        .iter()
                        .skip(i + 1)
                        .all(|scope| !scope.types.contains_key(n))
                    {
                        out.add(Arc::new(TypeWithOnlyName(n.clone())));
                        continue 'foreachtype;
                    }
                }
            }
            // no type alias
            if let Some(ty) = ty.simplify_for_display(info) {
                out.add_all(&ty);
            } else {
                out.add(Arc::clone(ty))
            }
        }
        out
    }
    pub fn simplified_as_string(&self, info: &crate::program::run::CheckInfo) -> String {
        self.simplify_for_display(info).with_info(info).to_string()
    }
}
impl Type {
    fn display(&self, info: &DisplayInfo, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.types.is_empty() {
            write!(f, "<unreachable>")
        } else {
            // if self.types.len() > 1 {
            //     write!(f, "{{")?;
            // }
            write!(f, "{}", self.types[0].with_display(info))?;
            for t in self.types.iter().skip(1) {
                write!(f, "/{}", t.with_display(info))?;
            }
            // if self.types.len() > 1 {
            //     write!(f, "}}")?;
            // }
            Ok(())
        }
    }
}
