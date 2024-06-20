use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use crate::{
    data::{self, Data, MersData, MersType, Type},
    errors::CheckError,
    parsing::{statements::to_string_literal, Source},
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// `thread: fn` turns `func /* () -> t */` into a `Thread`, which will run the function.
    /// `thread_finished: fn` returns `false` while the thread is running and `true` otherwise.
    /// `thread_await: fn` returns `t`, the value produced by the thread's function.
    pub fn with_multithreading(self) -> Self {
        self.add_type(
            "Thread".to_string(),
            Err(Arc::new(|s, i| {
                let mut src = Source::new_from_string_raw(s.to_owned());
                let srca = Arc::new(src.clone());
                let t = crate::parsing::types::parse_type(&mut src, &srca)?;
                Ok(Arc::new(Type::new(ThreadT(crate::parsing::types::type_from_parsed(&t, i)?))))
            })),
        )
        .add_var(
            "thread".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let mut out = Type::empty();
                    for t in a.types.iter() {
                        if let Some(f) = t.as_any().downcast_ref::<data::function::FunctionT>() {
                            match f.o(&Type::empty_tuple()) {
                                Ok(t) => out.add_all(&t),
                                Err(e) => return Err(CheckError::new().msg(format!("Can't call thread on a function which can't be called on an empty tuple: ")).err(e))
                            }
                        } else {
                            return Err(format!("thread: argument wasn't a function").into());
                        }
                    }
                    Ok(Type::new(ThreadT(out)))
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    if let Some(f) = a
                        .as_any()
                        .downcast_ref::<data::function::Function>()
                        .cloned()
                    {
                        Ok(Data::new(Thread(Arc::new(Mutex::new(Ok(std::thread::spawn(
                            move || f.run(Data::empty_tuple()),
                        )))))))
                    } else {
                        return Err("thread called, but arg wasn't a function".into());
                    }
                }),
                inner_statements: None,
            }),
        )
            .add_var("thread_finished".to_string(), Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    for t in a.types.iter() {
                        if !t.as_any().is::<ThreadT>() {
                            return Err(CheckError::new().msg(format!("Cannot call thread_finished on a value of type {t}, which isn't a thread but part of the argument {a}.")));
                        }
                    }
                    Ok(Type::new(data::bool::BoolT))
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let t = a.as_any().downcast_ref::<Thread>().unwrap().0.lock().unwrap();
                    Ok(Data::new(data::bool::Bool(match &*t {
                        Ok(t) => t.is_finished(),
                        Err(_d) => true,
                    })))
                }),
                inner_statements: None,
            }))
            .add_var("thread_await".to_string(), Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, _i| {
                    let mut out = Type::empty();
                    for t in a.types.iter() {
                        if let Some(t) = t.as_any().downcast_ref::<ThreadT>() {
                            out.add_all(&t.0);
                        } else {
                            return Err(CheckError::new().msg(format!("Cannot call thread_await on a value of type {t}, which isn't a thread but part of the argument {a}.")));
                        }
                    }
                    Ok(out)
                }),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    let mut t = a.as_any().downcast_ref::<Thread>().unwrap().0.lock().unwrap();
                    let d = match std::mem::replace(&mut *t, Err(Err(CheckError::new()))) {
                        Ok(t) => t.join().unwrap(),
                        Err(d) => d,
                    };
                    *t = Err(d.clone());
                    d
                }),
                inner_statements: None,
            }))
    }
}

#[derive(Clone)]
pub struct Thread(
    pub Arc<Mutex<Result<JoinHandle<Result<Data, CheckError>>, Result<Data, CheckError>>>>,
);
#[derive(Debug, Clone)]
pub struct ThreadT(pub Type);

impl MersData for Thread {
    fn is_eq(&self, _other: &dyn MersData) -> bool {
        false
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> Type {
        unreachable!("can't get type from Thread value! (can't construct Thread with syntax, so this should be fine?)")
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
impl MersType for ThreadT {
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.0.is_same_type_as(&other.0)
        } else {
            false
        }
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            self.0.is_included_in(&target.0)
        } else {
            false
        }
    }
    fn subtypes(&self, acc: &mut Type) {
        for t in self.0.subtypes_type().types {
            acc.add(Arc::new(Self(Type::newm(vec![t]))));
        }
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

impl Debug for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Thread>")
    }
}
impl Display for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Thread>")
    }
}
impl Display for ThreadT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Thread<{}>", to_string_literal(&self.0.to_string(), '>'))
    }
}
