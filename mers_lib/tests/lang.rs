use std::{fmt::Debug, sync::Arc};

use mers_lib::data::MersDataWInfo;
use mers_lib::prelude_compile::*;

use mers_lib::{
    data::{self, Data, Type},
    errors::CheckError,
};

#[test]
fn variable() -> Res {
    for n in -100..=100 {
        let n = n * n;
        assert_eq!(
            run_code(Config::new(), format!("x := {n}, x"))?,
            TypedData(
                Type::new(data::int::IntT),
                Data::new(data::int::Int(n as _)),
                mers_lib::info::Info::neverused(),
            )
        );
    }
    Ok(())
}

#[test]
fn mutating_a_variable() -> Res {
    assert_eq!(
        run_code(Config::new(), "x := 5, &x = 2, x")?,
        TypedData(
            Type::new(data::int::IntT),
            Data::new(data::int::Int(2)),
            mers_lib::info::Info::neverused()
        ),
    );
    Ok(())
}

#[test]
fn variable_shadowing() -> Res {
    assert_eq!(
        run_code(Config::new(), "x := 5, { x := 2, &x = 3 }, x")?,
        TypedData(
            Type::new(data::int::IntT),
            Data::new(data::int::Int(5)),
            mers_lib::info::Info::neverused()
        )
    );
    Ok(())
}

#[test]
fn identity_function() -> Res {
    assert_eq!(
        run_code(Config::new(), "id := x -> x, 4.id")?,
        TypedData(
            Type::new(data::int::IntT),
            Data::new(data::int::Int(4)),
            mers_lib::info::Info::neverused()
        )
    );
    Ok(())
}

type Res = Result<(), CheckError>;

fn run_code(cfg: Config, code: impl Into<String>) -> Result<TypedData, CheckError> {
    let mut src = Source::new_from_string(code.into());
    let srca = Arc::new(src.clone());
    let parsed = parse(&mut src, &srca)?;
    let (mut i1, mut i2, mut i3) = cfg.infos();
    let compiled = parsed.compile(&mut i1, Default::default())?;
    let output_type = compiled.check(&mut i3, Default::default())?;
    let output_data = compiled.run(&mut i2)?;
    Ok(TypedData(output_type, output_data, i2))
}

struct TypedData(Type, Data, mers_lib::program::run::Info);
impl Debug for TypedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Type: {}, Data: {}",
            self.0.with_info(&self.2),
            self.1.get().with_info(&self.2)
        )
    }
}
impl PartialEq for TypedData {
    fn eq(&self, other: &Self) -> bool {
        let t1 = self.0.is_same_type_as(&other.0);
        let t2 = other.0.is_same_type_as(&self.0);
        let d1 = self.1 == other.1;
        let d2 = other.1 == self.1;
        if t1 && !t2 {
            panic!("self is same type as other, but other is not same type as self (non-symmetrical eq)! self={}, other={}", self.0.with_info(&self.2), other.0.with_info(&self.2));
        }
        if t2 && !t1 {
            panic!("other is same type as self, but self is not same type as other (non-symmetrical eq)! other={}, self={}", other.0.with_info(&self.2), self.0.with_info(&self.2));
        }
        if d1 && !d2 {
            panic!("self is same data as other, but other is not same data as self (non-symmetrical eq)! self={}, other={}", self.1.get().with_info(&self.2), other.1.get().with_info(&self.2));
        }
        if d2 && !d1 {
            panic!("other is same data as self, but self is not same data as other (non-symmetrical eq)! other={}, self={}", other.1.get().with_info(&self.2), self.1.get().with_info(&self.2));
        }
        t1 && d1
    }
}
