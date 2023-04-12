use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};

use super::{
    block::RFunction,
    val_type::{VSingleType, VType},
};

#[derive(Clone, Debug, PartialEq)]
pub struct VData {
    // parents: Vec<()>,
    pub data: VDataEnum,
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
    EnumVariant(usize, Box<VData>),
}
impl PartialEq for VDataEnum {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Reference(a), Self::Reference(b)) => *a.lock().unwrap() == *b.lock().unwrap(),
            (Self::Reference(a), b) => a.lock().unwrap().data == *b,
            (a, Self::Reference(b)) => *a == b.lock().unwrap().data,
            (Self::Bool(a), Self::Bool(b)) => *a == *b,
            (Self::Int(a), Self::Int(b)) => *a == *b,
            (Self::Float(a), Self::Float(b)) => *a == *b,
            (Self::String(a), Self::String(b)) => *a == *b,
            (Self::Tuple(a), Self::Tuple(b)) | (Self::List(_, a), Self::List(_, b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
            }
            (Self::EnumVariant(a1, a2), Self::EnumVariant(b1, b2)) => *a1 == *b1 && *a2 == *b2,
            _ => false,
        }
    }
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
            VDataEnum::Function(f) => VSingleType::Function(f.input_output_map.clone()),
            VDataEnum::Thread(_, o) => VSingleType::Thread(o.clone()),
            VDataEnum::Reference(r) => r.lock().unwrap().out_single(),
            VDataEnum::EnumVariant(e, v) => VSingleType::EnumVariant(*e, v.out()),
        }
    }
    pub fn get(&self, i: usize) -> Option<Self> {
        self.data.get(i)
    }
    pub fn noenum(self) -> Self {
        self.data.noenum()
    }
}

impl VDataEnum {
    pub fn to(self) -> VData {
        VData { data: self }
    }
}

// get()
impl VDataEnum {
    pub fn noenum(self) -> VData {
        match self {
            Self::EnumVariant(_, v) => *v,
            v => v.to(),
        }
    }
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
            Self::EnumVariant(_, v) => v.get(i),
        }
    }
    pub fn matches_ref_bool(&self) -> bool {
        match self {
            VDataEnum::Tuple(v) => !v.is_empty(),
            VDataEnum::Bool(false) => false,
            _ => true,
        }
    }
    pub fn matches(self) -> Option<VData> {
        match self {
            VDataEnum::Tuple(mut tuple) => tuple.pop(),
            VDataEnum::Bool(v) => {
                if v {
                    Some(VDataEnum::Bool(v).to())
                } else {
                    None
                }
            }
            VDataEnum::EnumVariant(..) => None,
            other => Some(other.to()),
        }
    }
}
impl VSingleType {
    /// returns (can_fail_to_match, matches_as)
    pub fn matches(&self) -> (bool, VType) {
        match self {
            Self::Tuple(v) => match v.first() {
                Some(v) => (false, v.clone()),
                None => (true, VType { types: vec![] }),
            },
            Self::Bool => (true, Self::Bool.to()),
            Self::EnumVariant(..) | Self::EnumVariantS(..) => (true, VType { types: vec![] }),
            v => (false, v.clone().to()),
        }
    }
}
impl VType {
    /// returns (can_fail_to_match, matches_as)
    pub fn matches(&self) -> (bool, VType) {
        let mut can_fail = false;
        let mut matches_as = VType { types: vec![] };
        for t in self.types.iter() {
            let (f, t) = t.matches();
            can_fail |= f;
            matches_as = matches_as | t;
        }
        (can_fail, matches_as)
    }
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
                        VDataThreadEnum::Finished(VDataEnum::Bool(false).to()),
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
