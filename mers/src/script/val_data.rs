use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::{Arc, Mutex},
};

use super::{
    code_runnable::RFunction,
    global_info::{GSInfo, GlobalScriptInfo},
    val_type::{VSingleType, VType},
};

pub struct VData {
    /// (_, mutable) - if false, behave as CopyOnWrite.
    pub data: Arc<Mutex<(VDataEnum, bool)>>,
}
impl VData {
    /// if self is mutable, assigns the new value to the mutex.
    /// if self is immutable, creates a new mutex and sets self to mutable.
    pub fn assign(&mut self, new_val: VDataEnum) {
        {
            let mut d = self.data.lock().unwrap();
            if d.1 {
                d.0 = new_val;
                return;
            }
        }
        *self = new_val.to();
    }
    pub fn inner_replace(&mut self, new_val: VDataEnum) -> VDataEnum {
        {
            let mut d = self.data.lock().unwrap();
            if d.1 {
                return std::mem::replace(&mut d.0, new_val);
            }
        }
        let o = self.data().0.clone();
        *self = new_val.to();
        o
    }
    /// returns the contained VDataEnum. May or may not clone.
    pub fn inner(self) -> VDataEnum {
        self.data().0.clone()
    }
    /// ensures self is mutable, then returns a new instance of VData that is also mutable and uses the same Arc<Mutex<_>>.
    pub fn clone_mut(&mut self) -> Self {
        // if not mutable, copy and set to mutable.
        self.make_mut();
        // now, both self and the returned value are set to mutable and share the same mutex.
        self.clone_mut_assume()
    }
    /// like clone_mut, but assumes self is already mutable, and therefor does not need to mutate self
    /// as the Arc<Mutex<_>> will stay the same.
    pub fn clone_mut_assume(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
    pub fn ptr_eq(&self, rhs: &Self) -> bool {
        Arc::ptr_eq(&self.data, &rhs.data)
    }
    /// makes self mutable. might clone.
    pub fn make_mut(&mut self) -> &mut Self {
        {
            let mut s = self.data.lock().unwrap();
            if !s.1 {
                *s = (s.0.clone(), true);
            }
        }
        self
    }
    pub fn data(&self) -> std::sync::MutexGuard<(VDataEnum, bool)> {
        self.data.lock().unwrap()
    }
}
impl Clone for VData {
    fn clone(&self) -> Self {
        let mut d = self.data.lock().unwrap();
        // set to immutable, locking the data as-is.
        d.1 = false;
        // then return the same arc (-> avoid cloning)
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
impl Debug for VData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let d = self.data.lock().unwrap();
        if d.1 {
            write!(f, "(!mutable!):{:?}", d.0)
        } else {
            write!(f, "(immutable):{:?}", d.0)
        }
    }
}
impl PartialEq for VData {
    fn eq(&self, other: &Self) -> bool {
        self.data().0 == other.data().0
    }
}

#[derive(Debug)]
pub enum VDataEnum {
    Bool(bool),
    Int(isize),
    Float(f64),
    String(String),
    Tuple(Vec<VData>),
    List(VType, Vec<VData>),
    Function(RFunction),
    Thread(thread::VDataThread, VType),
    Reference(VData),
    EnumVariant(usize, Box<VData>),
}
impl Clone for VDataEnum {
    fn clone(&self) -> Self {
        match self {
            // exception: don't clone the value AND don't use CoW,
            // because we want to share the same Arc<Mutex<_>>.
            Self::Reference(r) => Self::Reference(r.clone_mut_assume()),
            // default impls
            Self::Bool(b) => Self::Bool(*b),
            Self::Int(i) => Self::Int(*i),
            Self::Float(f) => Self::Float(*f),
            Self::String(s) => Self::String(s.clone()),
            Self::Tuple(v) => Self::Tuple(v.clone()),
            Self::List(t, v) => Self::List(t.clone(), v.clone()),
            Self::Function(f) => Self::Function(f.clone()),
            Self::Thread(th, ty) => Self::Thread(th.clone(), ty.clone()),
            Self::EnumVariant(v, d) => Self::EnumVariant(v.clone(), d.clone()),
        }
    }
}
impl PartialEq for VDataEnum {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Reference(a), Self::Reference(b)) => a == b,
            (Self::Reference(a), b) => &a.data().0 == b,
            (a, Self::Reference(b)) => a == &b.data().0,
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
    pub fn safe_to_share(&self) -> bool {
        self.data().0.safe_to_share()
    }
    pub fn out(&self) -> VType {
        VType {
            types: vec![self.out_single()],
        }
    }
    pub fn out_single(&self) -> VSingleType {
        match &self.data().0 {
            VDataEnum::Bool(..) => VSingleType::Bool,
            VDataEnum::Int(..) => VSingleType::Int,
            VDataEnum::Float(..) => VSingleType::Float,
            VDataEnum::String(..) => VSingleType::String,
            VDataEnum::Tuple(v) => VSingleType::Tuple(v.iter().map(|v| v.out()).collect()),
            VDataEnum::List(t, _) => VSingleType::List(t.clone()),
            VDataEnum::Function(f) => VSingleType::Function(f.input_output_map.clone()),
            VDataEnum::Thread(_, o) => VSingleType::Thread(o.clone()),
            VDataEnum::Reference(r) => VSingleType::Reference(Box::new(r.out_single())),
            VDataEnum::EnumVariant(e, v) => VSingleType::EnumVariant(*e, v.out()),
        }
    }
    pub fn get(&self, i: usize) -> Option<Self> {
        self.data().0.get(i)
    }
    pub fn noenum(self) -> Self {
        self.inner().noenum()
    }
}

impl VDataEnum {
    pub fn to(self) -> VData {
        VData {
            data: Arc::new(Mutex::new((self, true))),
        }
    }
}

// get()
impl VDataEnum {
    pub fn safe_to_share(&self) -> bool {
        match self {
            Self::Bool(_) | Self::Int(_) | Self::Float(_) | Self::String(_) | Self::Function(_) => {
                true
            }
            Self::Tuple(v) | Self::List(_, v) => v.iter().all(|v| v.safe_to_share()),
            Self::Thread(..) | Self::Reference(..) | Self::EnumVariant(..) => false,
        }
    }
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
            Self::Reference(r) => r.get(i),
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

pub mod thread {
    use std::{
        fmt::Debug,
        sync::{Arc, Mutex},
        thread::JoinHandle,
        time::Duration,
    };

    use super::{VData, VDataEnum};

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
                VDataThreadEnum::Finished(v) => write!(f, "(thread finished)"),
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
}

//

pub struct VDataWInfo(VData, GSInfo);
impl Display for VDataWInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmtgs(f, Some(&self.1))
    }
}
impl VData {
    pub fn gsi(self, info: GSInfo) -> VDataWInfo {
        VDataWInfo(self, info)
    }
}

impl VDataEnum {
    pub fn fmtgs(&self, f: &mut Formatter, info: Option<&GlobalScriptInfo>) -> fmt::Result {
        match self {
            Self::Bool(true) => write!(f, "true"),
            Self::Bool(false) => write!(f, "false"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "\"{v}\""),
            Self::Tuple(v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    v.fmtgs(f, info)?;
                }
                write!(f, "]")
            }
            Self::List(_t, v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    v.fmtgs(f, info)?;
                    write!(f, " ")?;
                }
                write!(f, "...]")
            }
            Self::Function(func) => {
                VSingleType::Function(func.input_output_map.clone()).fmtgs(f, info)
            }
            Self::Thread(..) => write!(f, "[TODO] THREAD"),
            Self::Reference(inner) => {
                write!(f, "&")?;
                inner.fmtgs(f, info)
            }
            Self::EnumVariant(variant, inner) => {
                if let Some(name) = if let Some(info) = info {
                    info.enum_variants.iter().find_map(|(name, id)| {
                        if id == variant {
                            Some(name)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                } {
                    write!(f, "{name}: ")?;
                } else {
                    write!(f, "{variant}: ")?;
                }
                inner.fmtgs(f, info)
            }
        }
    }
}
impl Display for VDataEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None)
    }
}

impl VData {
    pub fn fmtgs(&self, f: &mut Formatter, info: Option<&GlobalScriptInfo>) -> fmt::Result {
        self.data().0.fmtgs(f, info)
    }
}
impl Display for VData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None)
    }
}
