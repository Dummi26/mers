use std::{marker::PhantomData, sync::Arc};

use crate::{
    data::{self, Data, MersData, Type},
    errors::CheckError,
};

use super::{FromMersData, ToMersData};

pub fn fun<I: FromMersData + 'static, O: ToMersData + 'static>(
    f: fn(I, &mut crate::program::run::Info) -> Result<O, CheckError>,
) -> impl StaticMersFunc {
    Box::new(f)
}
pub fn func<I: FromMersData + 'static, O: ToMersData + 'static>(
    f: fn(I, &mut crate::program::run::Info) -> Result<O, CheckError>,
) -> data::function::Function {
    Box::new(f).mers_func()
}
pub fn func_end<I: FromMersData + 'static>(
    f: fn(I, &mut crate::program::run::Info) -> !,
) -> data::function::Function {
    Box::new(f).mers_func()
}
pub fn func_err<I: FromMersData + 'static>(
    f: fn(I, &mut crate::program::run::Info) -> CheckError,
) -> data::function::Function {
    Box::new(f).mers_func()
}

// fn this_works() {
//     let cfg = ();
//     // type: `(Int -> Float, Byte -> Int)`
//     // todo: maybe add an Iterable<T> "type" that we can use like `[Itarable<Int>] (1, 2, 3)`
//     cfg.add_var(
//         "test".to_owned(),
//         Data::new(
//             TwoFuncs(
//                 OneFunc::new(|num: isize| Ok(num as f64 / 2.0)),
//                 OneFunc::new(|num: u8| Ok(num as isize)),
//             )
//             .to_mers_func(),
//         ),
//     );
// }

pub trait StaticMersFunc: Sized + 'static + Send + Sync {
    fn types() -> Vec<(Type, Type)>;
    fn run(
        &self,
        a: &(impl MersData + ?Sized),
        info: &mut crate::program::run::Info,
    ) -> Option<Result<Data, CheckError>>;
    fn mers_func(self) -> data::function::Function {
        data::function::Function::new_static(Self::types(), move |a, i| {
            match self.run(a.get().as_ref(), i) {
                Some(Ok(v)) => Ok(v),
                Some(Err(e)) => Err(e),
                None => Err(CheckError::from(format!(
                    "unexpected argument of type {}, expected {}",
                    a.get().as_type().with_info(i),
                    Type::new(data::function::FunctionT(
                        Err(Arc::new(Self::types())),
                        crate::info::Info::neverused()
                    ))
                    .with_info(i)
                ))),
            }
        })
    }
}

pub struct Funcs<A: StaticMersFunc, B: StaticMersFunc>(pub A, pub B);

pub trait Func: Send + Sync + 'static {
    type I: FromMersData;
    type O: ToMersData;
    fn run_func(
        &self,
        i: Self::I,
        info: &mut crate::program::run::Info,
    ) -> Result<Self::O, CheckError>;
}
impl<I: FromMersData + 'static, O: ToMersData + 'static> Func
    for fn(I, &mut crate::program::run::Info) -> Result<O, CheckError>
{
    type I = I;
    type O = O;
    fn run_func(
        &self,
        i: Self::I,
        info: &mut crate::program::run::Info,
    ) -> Result<Self::O, CheckError> {
        self(i, info)
    }
}

pub struct UnreachableDontConstruct(PhantomData<Self>);
impl ToMersData for UnreachableDontConstruct {
    fn as_type_to() -> Type {
        Type::empty()
    }
    fn represent(self) -> Data {
        unreachable!()
    }
}
impl<I: FromMersData + 'static> Func for fn(I, &mut crate::program::run::Info) -> ! {
    type I = I;
    type O = UnreachableDontConstruct;
    fn run_func(
        &self,
        i: Self::I,
        info: &mut crate::program::run::Info,
    ) -> Result<Self::O, CheckError> {
        self(i, info);
    }
}
impl<I: FromMersData + 'static> Func for fn(I, &mut crate::program::run::Info) -> CheckError {
    type I = I;
    type O = UnreachableDontConstruct;
    fn run_func(
        &self,
        i: Self::I,
        info: &mut crate::program::run::Info,
    ) -> Result<Self::O, CheckError> {
        Err(self(i, info))
    }
}

impl<F: Func + ?Sized> StaticMersFunc for Box<F> {
    fn types() -> Vec<(Type, Type)> {
        vec![(F::I::as_type_from(), F::O::as_type_to())]
    }
    fn run(
        &self,
        a: &(impl MersData + ?Sized),
        info: &mut crate::program::run::Info,
    ) -> Option<Result<Data, CheckError>> {
        F::I::try_represent(a, |v| {
            v.map(|v| self.run_func(v, info).map(|v| v.represent()))
        })
    }
}

impl<A: StaticMersFunc, B: StaticMersFunc> StaticMersFunc for Funcs<A, B> {
    fn types() -> Vec<(Type, Type)> {
        let mut o = A::types();
        for t in B::types() {
            if o.iter().any(|o| t.0.is_included_in(&o.0)) {
                // other function fully covers this case already,
                // ignore it in type signature as we will always call the first function instead of the 2nd one
            } else {
                o.push(t);
            }
        }
        o
    }
    fn run(
        &self,
        a: &(impl MersData + ?Sized),
        info: &mut crate::program::run::Info,
    ) -> Option<Result<Data, CheckError>> {
        self.0.run(a, info).or_else(|| self.1.run(a, info))
    }
}
