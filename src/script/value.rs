use std::{
    fmt::Debug,
    ops::BitOr,
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};

use super::block::RFunction;

#[derive(Clone, Debug)]
pub struct VData {
    // parents: Vec<()>,
    pub data: VDataEnum,
}
impl VData {
    pub fn out(&self) -> VType {
        VType {
            types: vec![self.out_single()],
        }
    }
    pub fn out_single(&self) -> VSingleType {
        match &self.data {
            VDataEnum::Bool(..) => VSingleType::Bool,
            VDataEnum::Int(..) => VSingleType::Int,
            VDataEnum::Float(..) => VSingleType::Float,
            VDataEnum::String(..) => VSingleType::String,
            VDataEnum::Tuple(v) => VSingleType::Tuple(v.iter().map(|v| v.out()).collect()),
            VDataEnum::List(t, _) => VSingleType::List(t.clone()),
            VDataEnum::Function(f) => VSingleType::Function(f.in_types().clone(), {
                eprintln!("Warn: generalizing function return type, disregarding input types. might make the type checker think it can return types it can only return with different arguments as the ones that were actually provided.");
                f.out_all()
            }),
            VDataEnum::Thread(_, o) => VSingleType::Thread(o.clone()),
            VDataEnum::Reference(r) => r.lock().unwrap().out_single(),
        }
    }
    pub fn get(&self, i: usize) -> Option<Self> {
        self.data.get(i)
    }
}

#[derive(Clone, Debug)]
pub enum VDataEnum {
    Bool(bool),
    Int(isize),
    Float(f64),
    String(String),
    Tuple(Vec<VData>),
    List(VType, Vec<VData>),
    Function(RFunction),
    Thread(VDataThread, VType),
    Reference(Arc<Mutex<VData>>),
}

#[derive(Clone)]
pub struct VDataThread(Arc<Mutex<VDataThreadEnum>>);
impl VDataThread {
    pub fn try_get(&self) -> Option<VData> {
        match &*self.lock() {
            VDataThreadEnum::Running(_) => None,
            VDataThreadEnum::Finished(v) => Some(v.clone()),
        }
    }
    pub fn get(&self) -> VData {
        let dur = Duration::from_millis(100);
        loop {
            match &*self.lock() {
                VDataThreadEnum::Running(v) => {
                    while !v.is_finished() {
                        std::thread::sleep(dur);
                    }
                }
                VDataThreadEnum::Finished(v) => return v.clone(),
            }
        }
    }
    pub fn lock(&self) -> std::sync::MutexGuard<VDataThreadEnum> {
        let mut mg = self.0.lock().unwrap();
        match &*mg {
            VDataThreadEnum::Running(v) => {
                if v.is_finished() {
                    let m = std::mem::replace(
                        &mut *mg,
                        VDataThreadEnum::Finished(VData {
                            data: VDataEnum::Bool(false),
                        }),
                    );
                    match m {
                        VDataThreadEnum::Running(v) => {
                            *mg = VDataThreadEnum::Finished(v.join().unwrap())
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => (),
        }
        mg
    }
}
impl Debug for VDataThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.lock() {
            VDataThreadEnum::Running(_) => write!(f, "(thread running)"),
            VDataThreadEnum::Finished(v) => write!(f, "(thread finished: {v})"),
        }
    }
}
pub enum VDataThreadEnum {
    Running(JoinHandle<VData>),
    Finished(VData),
}
impl VDataThreadEnum {
    pub fn to(self) -> VDataThread {
        VDataThread(Arc::new(Mutex::new(self)))
    }
}

impl VDataEnum {
    pub fn to(self) -> VData {
        VData { data: self }
    }
}

// get()
impl VDataEnum {
    pub fn get(&self, i: usize) -> Option<VData> {
        match self {
            Self::Bool(..)
            | Self::Int(..)
            | Self::Float(..)
            | Self::Function(..)
            | Self::Thread(..) => None,
            Self::String(s) => match s.chars().nth(i) {
                // Slow!
                Some(ch) => Some(Self::String(format!("{ch}")).to()),
                None => None,
            },
            Self::Tuple(v) | Self::List(_, v) => v.get(i).cloned(),
            Self::Reference(r) => r.lock().unwrap().get(i),
        }
    }
}
impl VSingleType {
    // None => Cannot get, Some(t) => getting can return t or nothing
    pub fn get(&self, i: usize) -> Option<VType> {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::Function(..) | Self::Thread(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => t.get(i).cloned(),
            Self::List(t) => Some(t.clone()),
            Self::Reference(r) => r.get(i),
        }
    }
}
impl VType {
    pub fn get(&self, i: usize) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get(i)?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VType {
    pub types: Vec<VSingleType>,
}
impl VType {
    /// Returns a vec with all types in self that aren't covered by rhs. If the returned vec is empty, self fits in rhs.
    pub fn fits_in(&self, rhs: &Self) -> Vec<VSingleType> {
        let mut no = vec![];
        for t in &self.types {
            // if t doesnt fit in any of rhs's types
            if !rhs.types.iter().any(|r| t.fits_in(r)) {
                no.push(t.clone())
            }
        }
        no
    }
    pub fn inner_types(&self) -> VType {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            for it in t.inner_types() {
                out = out | it.into();
            }
        }
        out
    }
}
impl BitOr for VType {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut types = self.types;
        for t in rhs.types {
            if !types.contains(&t) {
                types.push(t)
            }
        }
        Self { types }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VSingleType {
    Bool,
    Int,
    Float,
    String,
    Tuple(Vec<VType>),
    List(VType),
    Function(Vec<VType>, VType),
    Thread(VType),
    Reference(Box<Self>),
}
impl VSingleType {
    pub fn inner_types(&self) -> Vec<VSingleType> {
        match self {
            Self::Tuple(v) => {
                let mut types = vec![];
                for it in v {
                    // the tuple values
                    for it in &it.types {
                        // the possible types for each value
                        if !types.contains(it) {
                            types.push(it.clone());
                        }
                    }
                }
                types
            }
            Self::List(v) => v.types.clone(),
            _ => vec![],
        }
    }
    pub fn fits_in(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Bool, Self::Bool)
            | (Self::Int, Self::Int)
            | (Self::Float, Self::Float)
            | (Self::String, Self::String) => true,
            (Self::Bool | Self::Int | Self::Float | Self::String, _) => false,
            (Self::Tuple(a), Self::Tuple(b)) => {
                if a.len() == b.len() {
                    a.iter().zip(b.iter()).all(|(a, b)| a.fits_in(b).is_empty())
                } else {
                    false
                }
            }
            (Self::Tuple(_), _) => false,
            (Self::List(a), Self::List(b)) => a.fits_in(b).is_empty(),
            (Self::List(_), _) => false,
            (Self::Function(ai, ao), Self::Function(bi, bo)) => {
                ai.iter()
                    .zip(bi.iter())
                    .all(|(a, b)| a.fits_in(b).is_empty())
                    && ao.fits_in(bo).is_empty()
            }
            (Self::Function(..), _) => false,
            (Self::Thread(a), Self::Thread(b)) => a.fits_in(b).is_empty(),
            (Self::Thread(..), _) => false,
            (Self::Reference(r), Self::Reference(b)) => r.fits_in(b),
            (Self::Reference(_), _) => false,
        }
    }
}

impl Into<VType> for VSingleType {
    fn into(self) -> VType {
        VType { types: vec![self] }
    }
}
