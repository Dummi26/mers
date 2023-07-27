use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::{Arc, Mutex},
};

use super::{
    code_runnable::RFunction,
    fmtgs::FormatGs,
    global_info::{GSInfo, GlobalScriptInfo},
    val_type::{VSingleType, VType},
};

#[derive(Debug)]
pub enum VDataEnum {
    Bool(bool),
    Int(isize),
    Float(f64),
    String(String),
    Tuple(Vec<VData>),
    List(VType, Vec<VData>),
    Function(Arc<RFunction>),
    Thread(thread::VDataThread, VType),
    Reference(VData),
    EnumVariant(usize, Box<VData>),
}

#[cfg(not(debug_assertions))]
pub struct VData(Arc<Mutex<VDataInner>>);
#[cfg(debug_assertions)]
pub struct VData(pub Arc<Mutex<VDataInner>>, pub Option<String>);
pub enum VDataInner {
    Data(usize, Box<VDataEnum>),
    Mut(Arc<Mutex<VData>>),
    ClonedFrom(VData),
}
/// can be either Data, Mut or ClonedFrom.
/// - any ClonedFrom will point to a Data variant. It can never point to anything else.
///   it will increase the Data's clone count by one on creation and decrease it again on Drop::drop().
/// - any Mut will eventually point to a ClonedFrom or a Data variant. It can also point to another Mut.
impl VDataInner {
    fn to(self) -> VData {
        #[cfg(not(debug_assertions))]
        return VData(Arc::new(Mutex::new(self)));
        #[cfg(debug_assertions)]
        return VData(Arc::new(Mutex::new(self)), None);
    }
}
impl VDataEnum {
    pub fn to(self) -> VData {
        VDataInner::Data(0, Box::new(self)).to()
    }
}

impl VData {
    pub fn new_placeholder() -> Self {
        VDataEnum::Bool(false).to()
    }
    #[cfg(debug_assertions)]
    pub fn new_placeholder_with_name(name: String) -> Self {
        let mut o = VDataEnum::Bool(false).to();
        o.1 = Some(name);
        o
    }
    /// clones self, retrurning a new instance of self that will always yield the value self had when this function was called.
    /// note to dev: since the actual data is stored in VDataEnum, which either clones data or calls clone() (= clone_data()) on further VData, this automatically borrows all child data as immutable too. rust's Drop::drop() implementation (probably) handles everything for us too, so this can be implemented without thinking about recursion.
    pub fn clone_data(&self) -> Self {
        // TODO! implement CopyOnWrite. For now, just always copy. This also prevents mut references not existing since in ::Dat(cloned, _), cloned will always stay 0.
        return self.operate_on_data_immut(|v| v.clone()).to();
        // match &mut *self.0.lock().unwrap() {
        //     VDataInner::Data(cloned, _data) => {
        //         *cloned += 1;
        //         VDataInner::ClonedFrom(self.clone_arc()).to()
        //     }
        //     VDataInner::Mut(inner) => inner.lock().unwrap().clone_data(),
        //     VDataInner::ClonedFrom(inner) => inner.clone_data(),
        // }
    }
    /// clones self, returning a new instance of self that will always yield the same data as self, so that changes done to either are shared between both.
    pub fn clone_mut(&self) -> Self {
        VDataInner::Mut(Arc::new(Mutex::new(self.clone_arc()))).to()
    }
    fn clone_arc(&self) -> Self {
        #[cfg(not(debug_assertions))]
        return Self(Arc::clone(&self.0));
        #[cfg(debug_assertions)]
        return Self(Arc::clone(&self.0), self.1.clone());
    }
    pub fn operate_on_data_immut<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&VDataEnum) -> O,
    {
        match &*self.0.lock().unwrap() {
            VDataInner::Data(_, data) => func(data.as_ref()),
            VDataInner::Mut(inner) => inner.lock().unwrap().operate_on_data_immut(func),
            VDataInner::ClonedFrom(inner) => inner.operate_on_data_immut(func),
        }
    }
    /// runs func on the underlying data.
    /// attempts to get a mutable reference to the data. if this fails, it will (partially) clone the data, then point the VData to the new data,
    /// so that other VDatas pointing to the same original data aren't changed.
    pub fn operate_on_data_mut<F, O>(&mut self, func: F) -> O
    where
        F: FnOnce(&mut VDataEnum) -> O,
    {
        let (new_val, o) = {
            let mut lock = self.0.lock().unwrap();
            match &mut *lock {
                VDataInner::Data(count, data) => {
                    if *count == 0 {
                        (None, func(data.as_mut()))
                    } else {
                        let mut new_data = data.clone();
                        let o = func(new_data.as_mut());
                        // *self doesn't modify the ::Data, it instead points the value that wraps it to a new ::Data, leaving the old one as it was.
                        // for proof: data is untouched, only the new_data is ever modified.
                        let new_vdata = VDataInner::Data(0, new_data).to();
                        (Some(new_vdata), o)
                    }
                }
                VDataInner::Mut(inner) => (None, inner.lock().unwrap().operate_on_data_mut(func)),
                VDataInner::ClonedFrom(inner) => (None, inner.operate_on_data_mut(func)),
            }
        };
        if let Some(nv) = new_val {
            *self = nv;
        }
        o
    }

    /// Since operate_on_data_mut can clone, it may be inefficient for just assigning (where we don't care about the previous value, so it doesn't need to be cloned).
    /// This is what this function is for. (TODO: actually make it more efficient instead of using operate_on_data_mut)
    pub fn assign_data(&mut self, new_data: VDataEnum) {
        let o = self.operate_on_data_mut(|d| *d = new_data);
        o
    }
    /// Assigns the new_data to self. Affects all muts pointing to the same data, but no ClonedFroms.
    pub fn assign(&mut self, new: VData) {
        self.assign_data(new.inner_cloned())
        // !PROBLEM! If ClonedFrom always has to point to a Data, this may break things!
        // match &mut *self.0.lock().unwrap() {
        //     VDataInner::Data(count, data) => {
        //         // *self doesn't modify the ::Data, it instead points the value that wraps it to a new ::Data, leaving the old one as it was.
        //         // for proof: data is untouched.
        //         *self = new_data;
        //     }
        //     VDataInner::Mut(inner) => inner.lock().unwrap().assign(new_data),
        //     VDataInner::ClonedFrom(inner) => inner.assign(new_data),
        // }
    }
    /// assigns the value from self to assign_to if it's a reference, performs destructuring, and panics on invalid types that cannot be assigned to.
    pub fn assign_to(self: VData, mut assign_to: VData, info: &GSInfo) {
        // eprintln!("Assigning {self} to {assign_to}");
        assign_to.operate_on_data_mut(|assign_to| match assign_to {
            VDataEnum::Tuple(v) | VDataEnum::List(_, v) => {
                for (i, v) in v.iter().enumerate() {
                    self.get(i)
                        .expect(
                            "tried to assign to tuple, but value didn't return Some(_) on get()",
                        )
                        .assign_to(v.clone_data(), info)
                }
            }
            VDataEnum::Reference(r) => r.assign(self),
            o => todo!("ERR: Cannot assign to {o}."),
        })
    }
}
impl Drop for VDataInner {
    fn drop(&mut self) {
        if let Self::ClonedFrom(origin) = self {
            if let Self::Data(_ref_count, _data) = &mut *origin.0.lock().unwrap() {
                // *ref_count = ref_count.saturating_sub(1);
            }
        }
    }
}

impl VData {
    /// this will always clone! if a reference or mutable reference is enough, use operate_on_data_* instead!
    pub fn inner_cloned(&self) -> VDataEnum {
        self.operate_on_data_immut(|v| v.clone())
    }
}

// - - make VData act like VDataEnum (as if it were real data) - -

impl Clone for VData {
    fn clone(&self) -> Self {
        self.clone_data()
    }
}
impl FormatGs for VData {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        self.operate_on_data_immut(|v| v.fmtgs(f, info, form, file))
    }
}
impl Debug for VData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.operate_on_data_immut(|v| Debug::fmt(v, f))
    }
}
impl Display for VData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.operate_on_data_immut(|v| Display::fmt(v, f))
    }
}
impl PartialEq for VData {
    fn eq(&self, other: &Self) -> bool {
        self.operate_on_data_immut(|a| other.operate_on_data_immut(|b| a == b))
    }
}
impl PartialEq<VDataEnum> for VData {
    fn eq(&self, other: &VDataEnum) -> bool {
        self.operate_on_data_immut(|a| a == other)
    }
}
impl PartialEq<VData> for VDataEnum {
    fn eq(&self, other: &VData) -> bool {
        other.operate_on_data_immut(|b| self == b)
    }
}
impl VData {
    pub fn out_single(&self) -> VSingleType {
        self.operate_on_data_immut(|v| v.out_single())
    }
    pub fn out(&self) -> VType {
        self.out_single().to()
    }
    pub fn noenum(&self) -> Self {
        if let Some(v) = self.operate_on_data_immut(|v| v.noenum()) {
            v
        } else {
            self.clone_data()
        }
    }
    pub fn safe_to_share(&self) -> bool {
        self.operate_on_data_immut(|v| v.safe_to_share())
    }
    pub fn get(&self, i: usize) -> Option<VData> {
        self.operate_on_data_immut(|v| v.get(i))
    }
    pub fn get_ref(&mut self, i: usize) -> Option<VData> {
        self.operate_on_data_mut(|v| v.get_ref(i))
    }
    pub fn matches(&self) -> Option<Self> {
        match self.operate_on_data_immut(|v| v.matches()) {
            Some(Some(v)) => Some(v),
            Some(None) => Some(self.clone_data()),
            None => None,
        }
    }
    pub fn deref(&self) -> Option<Self> {
        self.operate_on_data_immut(|v| v.deref())
    }
}

// - - VDataEnum - -

impl Clone for VDataEnum {
    fn clone(&self) -> Self {
        match self {
            // exception: don't clone the value AND don't use CoW,
            // because we want to share the same Arc<Mutex<_>>.
            Self::Reference(r) => Self::Reference(r.clone_mut()),
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
            (Self::Reference(a), b) => a == b,
            (a, Self::Reference(b)) => a == b,
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

impl VDataEnum {
    pub fn deref(&self) -> Option<VData> {
        if let Self::Reference(r) = self {
            Some(r.clone_mut())
        } else {
            None
        }
    }
    pub fn out_single(&self) -> VSingleType {
        match self {
            Self::Bool(..) => VSingleType::Bool,
            Self::Int(..) => VSingleType::Int,
            Self::Float(..) => VSingleType::Float,
            Self::String(..) => VSingleType::String,
            Self::Tuple(v) => VSingleType::Tuple(v.iter().map(|v| v.out_single().to()).collect()),
            Self::List(t, _) => VSingleType::List(t.clone()),
            Self::Function(f) => VSingleType::Function(f.out_map.clone()),
            Self::Thread(_, o) => VSingleType::Thread(o.clone()),
            Self::Reference(r) => VSingleType::Reference(r.out()),
            Self::EnumVariant(e, v) => VSingleType::EnumVariant(*e, v.out_single().to()),
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
    pub fn noenum(&self) -> Option<VData> {
        match self {
            Self::EnumVariant(_, v) => Some(v.clone_data()),
            _v => None,
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
            Self::Reference(r) => r.clone_mut().get_ref(i),
            Self::EnumVariant(_, v) => v.get(i),
        }
    }
    /// this is guaranteed to return Self::Reference(_), if it returns Some(_).
    pub fn get_ref(&mut self, i: usize) -> Option<VData> {
        Some(Self::Reference(self.get_ref_inner(i)?).to())
    }
    pub fn get_ref_inner(&mut self, i: usize) -> Option<VData> {
        match self {
            Self::Bool(..)
            | Self::Int(..)
            | Self::Float(..)
            | Self::Function(..)
            | Self::Thread(..) => None,
            // TODO: String
            Self::String(_s) => None,
            Self::Tuple(v) | Self::List(_, v) => v.get(i).map(|v| v.clone_mut()),
            Self::Reference(r) => r.get_ref(i),
            Self::EnumVariant(_, v) => v.get_ref(i),
        }
    }
    /// Some(None) => matches with self
    pub fn matches(&self) -> Option<Option<VData>> {
        match self {
            VDataEnum::Tuple(tuple) => tuple.get(0).cloned().map(|v| Some(v)),
            VDataEnum::Bool(v) => {
                if *v {
                    Some(Some(VDataEnum::Bool(true).to()))
                } else {
                    None
                }
            }
            VDataEnum::EnumVariant(..) => None,
            _other => Some(None),
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
    pub fn matches(&self, info: &GlobalScriptInfo) -> (bool, VType) {
        let mut can_fail = false;
        let mut matches_as = VType { types: vec![] };
        for t in self.types.iter() {
            let (f, t) = t.matches();
            can_fail |= f;
            matches_as.add_types(t, info);
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
                VDataThreadEnum::Finished(_v) => write!(f, "(thread finished)"),
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
        self.0.fmtgs(
            f,
            Some(&self.1),
            &mut super::fmtgs::FormatInfo::default(),
            None,
        )
    }
}
impl VData {
    pub fn gsi(self, info: GSInfo) -> VDataWInfo {
        VDataWInfo(self, info)
    }
}

impl FormatGs for VDataEnum {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        match self {
            Self::Bool(true) => write!(f, "true"),
            Self::Bool(false) => write!(f, "false"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(v) => write!(
                f,
                "{}{}{}",
                form.value_string_quotes(info, "\"".to_owned()),
                form.value_string_content(info, v.to_owned()),
                form.value_string_quotes(info, "\"".to_owned())
            ),
            Self::Tuple(v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    v.fmtgs(f, info, form, file)?;
                }
                write!(f, "]")
            }
            Self::List(_t, v) => {
                write!(f, "[")?;
                for (_i, v) in v.iter().enumerate() {
                    v.fmtgs(f, info, form, file)?;
                    write!(f, " ")?;
                }
                write!(f, "...]")
            }
            Self::Function(func) => {
                VSingleType::Function(func.out_map.clone()).fmtgs(f, info, form, file)
            }
            Self::Thread(..) => write!(f, "[TODO] THREAD"),
            Self::Reference(inner) => {
                write!(f, "&")?;
                inner.fmtgs(f, info, form, file)
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
                inner.fmtgs(f, info, form, file)
            }
        }
    }
}
impl Display for VDataEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None, &mut super::fmtgs::FormatInfo::default(), None)
    }
}
