use std::sync::Arc;

use crate::{
    data::{self, Data, MersData, Type},
    errors::CheckError,
};

use super::{FromMersData, ToMersData};

pub fn func<I: FromMersData + 'static, O: ToMersData + 'static>(
    f: fn(I) -> Result<O, CheckError>,
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
    fn run(&self, a: &(impl MersData + ?Sized)) -> Option<Result<Data, CheckError>>;
    fn mers_func(self) -> data::function::Function {
        data::function::Function::new_static(Self::types(), move |a| {
            match self.run(a.get().as_ref()) {
                Some(Ok(v)) => Ok(v),
                Some(Err(e)) => Err(e),
                None => Err(CheckError::from(format!(
                    "unexpected argument of type {}, expected {}",
                    a.get().as_type(),
                    Type::new(data::function::FunctionT(Err(Arc::new(Self::types()))))
                ))),
            }
        })
    }
}

pub struct TwoFuncs<A: StaticMersFunc, B: StaticMersFunc>(pub A, pub B);

pub trait Func: Send + Sync + 'static {
    type I: FromMersData;
    type O: ToMersData;
    fn run_func(&self, i: Self::I) -> Result<Self::O, CheckError>;
}
impl<I: FromMersData + 'static, O: ToMersData + 'static> Func for fn(I) -> Result<O, CheckError> {
    type I = I;
    type O = O;
    fn run_func(&self, i: Self::I) -> Result<Self::O, CheckError> {
        self(i)
    }
}

impl<F: Func + ?Sized> StaticMersFunc for Box<F> {
    fn types() -> Vec<(Type, Type)> {
        vec![(F::I::as_type_from(), F::O::as_type_to())]
    }
    fn run(&self, a: &(impl MersData + ?Sized)) -> Option<Result<Data, CheckError>> {
        F::I::try_represent(a, |v| v.map(|v| self.run_func(v).map(|v| v.represent())))
    }
}

impl<A: StaticMersFunc, B: StaticMersFunc> StaticMersFunc for TwoFuncs<A, B> {
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
    fn run(&self, a: &(impl MersData + ?Sized)) -> Option<Result<Data, CheckError>> {
        self.0.run(a).or_else(|| self.1.run(a))
    }
}
