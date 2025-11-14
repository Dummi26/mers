pub mod function;

use std::sync::Arc;

use crate::data::{self, bool::bool_type, Data, MersData, Type};

pub trait FromMersData: Sized {
    fn as_type_from() -> Type;
    fn can_represent(t: &Type) -> bool;
    /// **NOTE: `f` may only used the passed value of type `Self` during the call to `f`**.
    /// Storing the value anywhere, moving it to a thread or otherwise assuming that it lives longer than the function call
    /// violates Rust's lifetime rules, but is allowed by the compiler because of some `unsafe` code which
    /// calls `f` with a `Self` type with a lifetime that is incorrect on purpose (but seemingly necessary for this to work at all).
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O;
}
pub trait ToMersData {
    /// what type this will become when `represent` is called
    fn as_type_to() -> Type;
    fn represent(self) -> Data;
}

/// Generates types like `(A)/()`. If `A` is a tuple-type, consider using `AnyOrNone` instead.
pub struct OneOrNone<A>(pub Option<A>);
/// Mainly used to generate types like `(A, B)/()` where `OneOrNone` would generate `((A, B))/()`, but can also generate `A/()`
pub struct AnyOrNone<A>(pub Option<A>);
impl<A: FromMersData> FromMersData for OneOrNone<A> {
    fn as_type_from() -> Type {
        Type::newm(vec![
            Arc::new(data::tuple::TupleT(vec![A::as_type_from()])),
            Arc::new(data::tuple::TupleT(vec![])),
        ])
    }
    fn can_represent(t: &Type) -> bool {
        t.one_tuple_possible_content()
            .is_some_and(|t| A::can_represent(&t))
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d
            .as_any()
            .downcast_ref::<data::tuple::Tuple>()
            .filter(|v| v.0.len() <= 1)
        {
            if v.0.is_empty() {
                f(Some(Self(None)))
            } else {
                A::try_represent(v.0[0].read().get().as_ref(), |v1| {
                    if let Some(va) = v1 {
                        f(Some(Self(Some(va))))
                    } else {
                        f(None)
                    }
                })
            }
        } else {
            f(None)
        }
    }
}
impl<A: ToMersData> ToMersData for OneOrNone<A> {
    fn as_type_to() -> Type {
        Type::newm(vec![
            Arc::new(data::tuple::TupleT(vec![A::as_type_to()])),
            Arc::new(data::tuple::TupleT(vec![])),
        ])
    }
    fn represent(self) -> Data {
        if let Some(v) = self.0 {
            Data::one_tuple(v.represent())
        } else {
            Data::empty_tuple()
        }
    }
}
impl<A: FromMersData> FromMersData for AnyOrNone<A> {
    fn as_type_from() -> Type {
        Type::newm(vec![
            Arc::new(data::tuple::TupleT(vec![A::as_type_from()])),
            Arc::new(data::tuple::TupleT(vec![])),
        ])
    }
    fn can_represent(t: &Type) -> bool {
        t.one_tuple_possible_content()
            .is_some_and(|t| A::can_represent(&t))
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if d.as_any()
            .downcast_ref::<data::tuple::Tuple>()
            .is_some_and(|v| v.0.is_empty())
        {
            f(Some(Self(None)))
        } else {
            A::try_represent(d, |v1| {
                if let Some(va) = v1 {
                    f(Some(Self(Some(va))))
                } else {
                    f(None)
                }
            })
        }
    }
}
impl<A: ToMersData> ToMersData for AnyOrNone<A> {
    fn as_type_to() -> Type {
        let mut o = A::as_type_to();
        o.add_all(&Type::empty_tuple());
        o
    }
    fn represent(self) -> Data {
        if let Some(v) = self.0 {
            v.represent()
        } else {
            Data::empty_tuple()
        }
    }
}

pub enum OneOf<A, B> {
    A(A),
    B(B),
}
impl<A: FromMersData, B: FromMersData> FromMersData for OneOf<A, B> {
    fn as_type_from() -> Type {
        let mut o = A::as_type_from();
        o.add_all(&B::as_type_from());
        o
    }
    fn can_represent(t: &Type) -> bool {
        A::can_represent(t) || B::can_represent(t)
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        A::try_represent(d, |v| {
            if let Some(v) = v {
                f(Some(OneOf::A(v)))
            } else {
                B::try_represent(d, |v| {
                    if let Some(v) = v {
                        f(Some(OneOf::B(v)))
                    } else {
                        f(None)
                    }
                })
            }
        })
    }
}
impl<A: ToMersData, B: ToMersData> ToMersData for OneOf<A, B> {
    fn as_type_to() -> Type {
        let mut o = A::as_type_to();
        o.add_all(&B::as_type_to());
        o
    }
    fn represent(self) -> Data {
        match self {
            Self::A(v) => v.represent(),
            Self::B(v) => v.represent(),
        }
    }
}

pub struct IterToList<T: ToMersData, I: Iterator<Item = T>>(pub I);
impl<T: ToMersData, I: Iterator<Item = T>> ToMersData for IterToList<T, I> {
    fn as_type_to() -> Type {
        Type::new(super::with_list::ListT(T::as_type_to()))
    }
    fn represent(self) -> Data {
        Data::new(super::with_list::List(
            self.0.map(|v| v.represent()).collect(),
        ))
    }
}

impl FromMersData for () {
    fn as_type_from() -> Type {
        Self::as_type_to()
    }
    fn can_represent(t: &Type) -> bool {
        t.is_zero_tuple()
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        f(d.as_any()
            .downcast_ref::<data::tuple::Tuple>()
            .is_some_and(|v| v.0.is_empty())
            .then_some(()))
    }
}
impl ToMersData for () {
    fn as_type_to() -> Type {
        Type::empty_tuple()
    }
    fn represent(self) -> Data {
        Data::empty_tuple()
    }
}

impl<A: FromMersData> FromMersData for (A,) {
    fn as_type_from() -> Type {
        Type::new(data::tuple::TupleT(vec![A::as_type_from()]))
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Self::as_type_from())
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d
            .as_any()
            .downcast_ref::<data::tuple::Tuple>()
            .filter(|v| v.0.len() == 1)
        {
            A::try_represent(v.0[0].read().get().as_ref(), |v1| {
                if let Some(va) = v1 {
                    f(Some((va,)))
                } else {
                    f(None)
                }
            })
        } else {
            f(None)
        }
    }
}
impl<A: ToMersData> ToMersData for (A,) {
    fn as_type_to() -> Type {
        Type::new(data::tuple::TupleT(vec![A::as_type_to()]))
    }
    fn represent(self) -> Data {
        Data::new(data::tuple::Tuple::from([self.0.represent()]))
    }
}

impl<A: FromMersData, B: FromMersData> FromMersData for (A, B) {
    fn as_type_from() -> Type {
        Type::new(data::tuple::TupleT(vec![
            A::as_type_from(),
            B::as_type_from(),
        ]))
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Self::as_type_from())
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d
            .as_any()
            .downcast_ref::<data::tuple::Tuple>()
            .filter(|v| v.0.len() == 2)
        {
            A::try_represent(v.0[0].read().get().as_ref(), |v1| {
                if let Some(va) = v1 {
                    B::try_represent(v.0[1].read().get().as_ref(), |v2| {
                        if let Some(vb) = v2 {
                            f(Some((va, vb)))
                        } else {
                            f(None)
                        }
                    })
                } else {
                    f(None)
                }
            })
        } else {
            f(None)
        }
    }
}
impl<A: ToMersData, B: ToMersData> ToMersData for (A, B) {
    fn as_type_to() -> Type {
        Type::new(data::tuple::TupleT(vec![A::as_type_to(), B::as_type_to()]))
    }
    fn represent(self) -> Data {
        Data::new(data::tuple::Tuple::from([
            self.0.represent(),
            self.1.represent(),
        ]))
    }
}
impl<A: FromMersData, B: FromMersData, C: FromMersData> FromMersData for (A, B, C) {
    fn as_type_from() -> Type {
        Type::new(data::tuple::TupleT(vec![
            A::as_type_from(),
            B::as_type_from(),
            C::as_type_from(),
        ]))
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Self::as_type_from())
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d
            .as_any()
            .downcast_ref::<data::tuple::Tuple>()
            .filter(|v| v.0.len() == 3)
        {
            A::try_represent(v.0[0].read().get().as_ref(), |v1| {
                if let Some(va) = v1 {
                    B::try_represent(v.0[1].read().get().as_ref(), |v2| {
                        if let Some(vb) = v2 {
                            C::try_represent(v.0[2].read().get().as_ref(), |v3| {
                                if let Some(vc) = v3 {
                                    f(Some((va, vb, vc)))
                                } else {
                                    f(None)
                                }
                            })
                        } else {
                            f(None)
                        }
                    })
                } else {
                    f(None)
                }
            })
        } else {
            f(None)
        }
    }
}
impl<A: ToMersData, B: ToMersData, C: ToMersData> ToMersData for (A, B, C) {
    fn as_type_to() -> Type {
        Type::new(data::tuple::TupleT(vec![
            A::as_type_to(),
            B::as_type_to(),
            C::as_type_to(),
        ]))
    }
    fn represent(self) -> Data {
        Data::new(data::tuple::Tuple::from([
            self.0.represent(),
            self.1.represent(),
            self.2.represent(),
        ]))
    }
}

impl FromMersData for bool {
    fn as_type_from() -> Type {
        Self::as_type_to()
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&bool_type())
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d.as_any().downcast_ref::<data::bool::Bool>() {
            f(Some(v.0))
        } else {
            f(None)
        }
    }
}
impl ToMersData for bool {
    fn as_type_to() -> Type {
        bool_type()
    }
    fn represent(self) -> Data {
        Data::new(data::bool::Bool(self))
    }
}

impl FromMersData for u8 {
    fn as_type_from() -> Type {
        Self::as_type_to()
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Type::new(data::byte::ByteT))
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d.as_any().downcast_ref::<data::byte::Byte>() {
            f(Some(v.0))
        } else {
            f(None)
        }
    }
}
impl ToMersData for u8 {
    fn as_type_to() -> Type {
        Type::new(data::byte::ByteT)
    }
    fn represent(self) -> Data {
        Data::new(data::byte::Byte(self))
    }
}

/// An integer within the range `N..=M`
pub struct IntR<const N: isize, const M: isize>(pub isize);
impl<const N: isize, const M: isize> FromMersData for IntR<N, M> {
    fn as_type_from() -> Type {
        Self::as_type_to()
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Type::new(data::int::IntT(N, M)))
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d.as_any().downcast_ref::<data::int::Int>() {
            if N <= v.0 && v.0 <= M {
                f(Some(Self(v.0)))
            } else {
                f(None)
            }
        } else {
            f(None)
        }
    }
}
impl<const N: isize, const M: isize> ToMersData for IntR<N, M> {
    fn as_type_to() -> Type {
        Type::new(data::int::IntT(N, M))
    }
    fn represent(self) -> Data {
        Data::new(data::int::Int(self.0))
    }
}

impl FromMersData for f64 {
    fn as_type_from() -> Type {
        Self::as_type_to()
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Type::new(data::float::FloatT))
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d.as_any().downcast_ref::<data::float::Float>() {
            f(Some(v.0))
        } else {
            f(None)
        }
    }
}
impl ToMersData for f64 {
    fn as_type_to() -> Type {
        Type::new(data::float::FloatT)
    }
    fn represent(self) -> Data {
        Data::new(data::float::Float(self))
    }
}

impl FromMersData for &str {
    fn as_type_from() -> Type {
        String::as_type_to()
    }
    fn can_represent(t: &Type) -> bool {
        t.is_included_in(&Type::new(data::string::StringT))
    }
    fn try_represent<O, F: FnOnce(Option<Self>) -> O>(d: &(impl MersData + ?Sized), f: F) -> O {
        if let Some(v) = d.as_any().downcast_ref::<data::string::String>() {
            let v = v.0.as_str();
            unsafe { f(Some(std::ptr::from_ref(v).as_ref().unwrap())) }
        } else {
            f(None)
        }
    }
}
impl ToMersData for String {
    fn as_type_to() -> Type {
        Type::new(data::string::StringT)
    }
    fn represent(self) -> Data {
        Data::new(data::string::String(self))
    }
}
