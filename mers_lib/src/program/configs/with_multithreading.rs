use std::{
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use crate::{
    data::{self, Data, MersData, MersType, Type},
    program::{self, run::CheckInfo},
};

use super::Config;

impl Config {
    /// `thread: fn` turns `(func, arg)` into a `Thread`, which will run the function with the argument.
    /// `thread_get_result: fn` returns `()` while the thread is running and `(result)` otherwise.
    pub fn with_multithreading(self) -> Self {
        self.add_type(
            "Thread".to_string(),
            Type::new(ThreadT(Type::empty_tuple())),
        )
        .add_var(
            "thread".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, i| todo!()),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    if let (Some(f), Some(arg)) = (
                        a.get(0).and_then(|v| {
                            v.get()
                                .as_any()
                                .downcast_ref::<data::function::Function>()
                                .cloned()
                        }),
                        a.get(1),
                    ) {
                        Data::new(Thread(Arc::new(Mutex::new(Ok(std::thread::spawn(
                            move || f.run(arg),
                        ))))))
                    } else {
                        unreachable!("thread called, but arg wasn't a (function, _)");
                    }
                }),
            }),
        )
        .add_var(
            "thread_get_result".to_string(),
            Data::new(data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(|a, i| todo!()),
                run: Arc::new(|a, _i| {
                    let a = a.get();
                    if let Some(t) = a
                        .get(0)
                        .and_then(|v| v.get().as_any().downcast_ref::<Thread>().cloned())
                    {
                        let mut t = t.0.lock().unwrap();
                        if t.as_ref().is_ok_and(|t| t.is_finished()) {
                            unsafe {
                                // extract the JoinHandle from the Result by replacing it with uninitialized memory.
                                #[allow(invalid_value)]
                                let thread = std::mem::replace(
                                    &mut *t,
                                    std::mem::MaybeUninit::uninit().assume_init(),
                                )
                                .unwrap();
                                // forget about t and its uninitialized memory while replacing it with the new value
                                std::mem::forget(std::mem::replace(
                                    &mut *t,
                                    Err(thread.join().unwrap()),
                                ));
                            }
                        }
                        match &*t {
                            Ok(_) => Data::empty_tuple(),
                            Err(v) => Data::one_tuple(v.clone()),
                        }
                    } else {
                        unreachable!("thread_get_result called, but arg wasn't a Thread");
                    }
                }),
            }),
        )
    }
}

#[derive(Clone)]
pub struct Thread(Arc<Mutex<Result<JoinHandle<Data>, Data>>>);
#[derive(Debug)]
pub struct ThreadT(Type);

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
    fn is_included_in_single(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            self.0.is_included_in_single(&target.0)
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
        write!(f, "<Thread>")
    }
}
