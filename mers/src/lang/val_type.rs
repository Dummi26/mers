use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    ops::BitOr,
};

use super::{
    fmtgs::FormatGs,
    global_info::{self, GSInfo, GlobalScriptInfo},
};

use super::global_info::LogMsg;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VType {
    pub types: Vec<VSingleType>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VSingleType {
    Bool,
    Int,
    Float,
    String,
    Tuple(Vec<VType>),
    List(VType),
    Function(Vec<(Vec<VSingleType>, VType)>),
    Thread(VType),
    Reference(Box<Self>),
    EnumVariant(usize, VType),
    EnumVariantS(String, VType),
    CustomType(usize),
    CustomTypeS(String),
}

impl VSingleType {
    /// None => Cannot get, Some(t) => getting can return t or nothing
    pub fn get(&self, i: usize, gsinfo: &GlobalScriptInfo) -> Option<VType> {
        match self {
            Self::Bool
            | Self::Int
            | Self::Float
            | Self::Function(..)
            | Self::Thread(..)
            | Self::EnumVariant(..)
            | Self::EnumVariantS(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => t.get(i).cloned(),
            Self::List(t) => Some(t.clone()),
            Self::Reference(r) => r.get_ref(i, gsinfo),
            Self::CustomType(t) => gsinfo.custom_types[*t].get(i, gsinfo),
            &Self::CustomTypeS(_) => {
                unreachable!("CustomTypeS instead of CustomType, compiler bug? [get]")
            }
        }
    }
    pub fn get_ref(&self, i: usize, gsinfo: &GlobalScriptInfo) -> Option<VType> {
        match self {
            Self::Bool
            | Self::Int
            | Self::Float
            | Self::Function(..)
            | Self::Thread(..)
            | Self::EnumVariant(..)
            | Self::EnumVariantS(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => t.get(i).map(|v| v.reference()),
            Self::List(t) => Some(t.reference()),
            Self::Reference(r) => r.get_ref(i, gsinfo),
            Self::CustomType(t) => Some(gsinfo.custom_types[*t].get(i, gsinfo)?.reference()),
            &Self::CustomTypeS(_) => {
                unreachable!("CustomTypeS instead of CustomType, compiler bug? [get]")
            }
        }
    }
    /// None => might not always return t, Some(t) => can only return t
    pub fn get_always(&self, i: usize, info: &GlobalScriptInfo) -> Option<VType> {
        match self {
            Self::Bool
            | Self::Int
            | Self::Float
            | Self::String
            | Self::List(_)
            | Self::Function(..)
            | Self::Thread(..)
            | Self::EnumVariant(..)
            | Self::EnumVariantS(..) => None,
            Self::Tuple(t) => t.get(i).cloned(),
            Self::Reference(r) => r.get_always_ref(i, info),
            Self::CustomType(t) => info.custom_types[*t].get_always(i, info),
            Self::CustomTypeS(_) => {
                unreachable!("CustomTypeS instead of CustomType, compiler bug? [get_always]")
            }
        }
    }
    pub fn get_always_ref(&self, i: usize, info: &GlobalScriptInfo) -> Option<VType> {
        match self {
            Self::Bool
            | Self::Int
            | Self::Float
            | Self::String
            | Self::List(_)
            | Self::Function(..)
            | Self::Thread(..)
            | Self::EnumVariant(..)
            | Self::EnumVariantS(..) => None,
            Self::Tuple(t) => Some(t.get(i)?.reference()),
            Self::Reference(r) => r.get_always_ref(i, info),
            Self::CustomType(t) => info.custom_types[*t].get_always_ref(i, info),
            Self::CustomTypeS(_) => {
                unreachable!("CustomTypeS instead of CustomType, compiler bug? [get_always]")
            }
        }
    }
}
impl VType {
    pub fn empty() -> Self {
        Self { types: vec![] }
    }
    pub fn get(&self, i: usize, info: &GlobalScriptInfo) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get(i, info)?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
    pub fn get_always(&self, i: usize, info: &GlobalScriptInfo) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get_always(i, info)?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
    pub fn get_always_ref(&self, i: usize, info: &GlobalScriptInfo) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get_always_ref(i, info)?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
    /// returns Some(true) or Some(false) if all types are references or not references. If it is mixed or types is empty, returns None.
    pub fn is_reference(&self) -> Option<bool> {
        let mut noref = false;
        let mut reference = false;
        for t in &self.types {
            if t.is_reference() {
                reference = true;
            } else {
                noref = true;
            }
        }
        if noref != reference {
            Some(reference)
        } else {
            // either empty (false == false) or mixed (true == true)
            None
        }
    }
    /// returns Some(t) where t is the type you get from dereferencing self or None if self contains even a single type that cannot be dereferenced.
    pub fn dereference(&self) -> Option<Self> {
        let mut out = Self::empty();
        for t in self.types.iter() {
            out = out | t.deref()?.to();
        }
        Some(out)
    }
    pub fn reference(&self) -> Self {
        let mut out = Self::empty();
        Self {
            types: self
                .types
                .iter()
                .map(|v| VSingleType::Reference(Box::new(v.clone())))
                .collect(),
        }
    }
}

impl VSingleType {
    pub fn get_any(&self, info: &GlobalScriptInfo) -> Option<VType> {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::Function(..) | Self::Thread(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => Some(t.iter().fold(VType { types: vec![] }, |a, b| a | b)),
            Self::List(t) => Some(t.clone()),
            Self::Reference(r) => r.get_any_ref(info),
            Self::EnumVariant(_, t) => t.get_any(info),
            Self::EnumVariantS(..) => unreachable!(),
            Self::CustomType(t) => info.custom_types[*t].get_any(info),
            Self::CustomTypeS(_) => unreachable!(),
        }
    }
    pub fn get_any_ref(&self, info: &GlobalScriptInfo) -> Option<VType> {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::Function(..) | Self::Thread(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => Some(
                t.iter()
                    .fold(VType { types: vec![] }, |a, b| a | b.reference()),
            ),
            Self::List(t) => Some(t.reference()),
            // TODO: idk if this is right...
            Self::Reference(r) => r.get_any_ref(info),
            Self::EnumVariant(_, t) => t.get_any_ref(info),
            Self::EnumVariantS(..) => unreachable!(),
            Self::CustomType(t) => info.custom_types[*t].get_any(info),
            Self::CustomTypeS(_) => unreachable!(),
        }
    }
    pub fn is_reference(&self) -> bool {
        match self {
            Self::Reference(_) => true,
            _ => false,
        }
    }
    pub fn deref(&self) -> Option<VSingleType> {
        if let Self::Reference(v) = self {
            Some(*v.clone())
        } else {
            None
        }
    }
}
impl VType {
    pub fn get_any(&self, info: &GlobalScriptInfo) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get_any(info)?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
    pub fn get_any_ref(&self, info: &GlobalScriptInfo) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get_any_ref(info)?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
}

impl VType {
    /// Returns a vec with all types in self that aren't covered by rhs. If the returned vec is empty, self fits in rhs.
    pub fn fits_in(&self, rhs: &Self, info: &GlobalScriptInfo) -> Vec<VSingleType> {
        let mut no = vec![];
        for t in &self.types {
            // if t doesnt fit in any of rhs's types
            if !t.fits_in_type(rhs, info) {
                no.push(t.clone())
            }
        }
        if info.log.vtype_fits_in.log() {
            info.log
                .log(LogMsg::VTypeFitsIn(self.clone(), rhs.clone(), no.clone()))
        }
        no
    }
    pub fn inner_types(&self) -> VType {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            for it in t.inner_types() {
                out = out | it.to();
            }
        }
        out
    }
    pub fn enum_variants(&mut self, enum_variants: &mut HashMap<String, usize>) {
        for t in &mut self.types {
            t.enum_variants(enum_variants);
        }
    }
    pub fn contains(&self, t: &VSingleType, info: &GlobalScriptInfo) -> bool {
        t.fits_in_type(self, info)
    }
    pub fn noenum(self) -> Self {
        let mut o = Self { types: vec![] };
        for t in self.types {
            o = o | t.noenum();
        }
        o
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
impl BitOr<&VType> for VType {
    type Output = Self;
    fn bitor(self, rhs: &Self) -> Self::Output {
        let mut types = self.types;
        for t in &rhs.types {
            if !types.contains(t) {
                types.push(t.clone())
            }
        }
        Self { types }
    }
}

impl VSingleType {
    pub fn to(self) -> VType {
        VType { types: vec![self] }
    }
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
            // NOTE: to make ints work in for loops
            Self::Int => vec![Self::Int],
            // for iterators in for loops: the match of the function's returned value make up the inner type
            Self::Function(f) => {
                // function that takes no inputs
                if let Some(out) = f.iter().find_map(|(args, out)| {
                    if args.is_empty() {
                        Some(out.clone().inner_types())
                    } else {
                        None
                    }
                }) {
                    out.types
                } else {
                    vec![]
                }
            }
            Self::Reference(r) => r.inner_types_ref(),
            _ => vec![],
        }
    }
    pub fn inner_types_ref(&self) -> Vec<VSingleType> {
        match self {
            Self::Tuple(v) => {
                let mut types = vec![];
                for it in v {
                    // the tuple values
                    for it in &it.types {
                        // the possible types for each value
                        if !types.contains(it) {
                            types.push(Self::Reference(Box::new(it.clone())));
                        }
                    }
                }
                types
            }
            Self::List(v) => v
                .types
                .iter()
                .map(|v| Self::Reference(Box::new(v.clone())))
                .collect(),
            Self::Reference(r) => r.inner_types_ref(),
            _ => vec![],
        }
    }
    pub fn noenum(self) -> VType {
        match self {
            Self::EnumVariant(_, v) | Self::EnumVariantS(_, v) => v,
            v => v.to(),
        }
    }
    /// converts all Self::EnumVariantS to Self::EnumVariant
    pub fn enum_variants(&mut self, enum_variants: &mut HashMap<String, usize>) {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::String => (),
            Self::Tuple(v) => {
                for t in v {
                    t.enum_variants(enum_variants);
                }
            }
            Self::List(t) => t.enum_variants(enum_variants),
            Self::Function(f) => {
                for f in f {
                    for t in &mut f.0 {
                        t.enum_variants(enum_variants);
                    }
                    f.1.enum_variants(enum_variants);
                }
            }
            Self::Thread(v) => v.enum_variants(enum_variants),
            Self::Reference(v) => v.enum_variants(enum_variants),
            Self::EnumVariant(_e, v) => v.enum_variants(enum_variants),
            Self::EnumVariantS(e, v) => {
                let e = if let Some(e) = enum_variants.get(e) {
                    *e
                } else {
                    let v = enum_variants.len();
                    enum_variants.insert(e.clone(), v);
                    v
                };
                v.enum_variants(enum_variants);
                *self = Self::EnumVariant(e, v.clone());
            }
            Self::CustomType(_) | Self::CustomTypeS(_) => (),
        }
    }
    pub fn fits_in(&self, rhs: &Self, info: &GlobalScriptInfo) -> bool {
        let o = match (self, rhs) {
            (Self::Reference(r), Self::Reference(b)) => r.fits_in(b, info),
            (Self::Reference(_), _) | (_, Self::Reference(_)) => false,
            (Self::EnumVariant(v1, t1), Self::EnumVariant(v2, t2)) => {
                *v1 == *v2 && t1.fits_in(&t2, info).is_empty()
            }
            (Self::CustomType(a), Self::CustomType(b)) => *a == *b, /* || info.custom_types[*a].fits_in(&info.custom_types[*b], info).is_empty() */
            (Self::CustomType(a), b) => info.custom_types[*a]
                .fits_in(&b.clone().to(), info)
                .is_empty(),
            (a, Self::CustomType(b)) => a
                .clone()
                .to()
                .fits_in(&info.custom_types[*b], info)
                .is_empty(),
            (Self::CustomTypeS(_), _) | (_, Self::CustomTypeS(_)) => {
                unreachable!("CustomTypeS instead of CustomType - compiler bug?")
            }
            (Self::EnumVariant(..), _) | (_, Self::EnumVariant(..)) => false,
            (Self::EnumVariantS(..), _) | (_, Self::EnumVariantS(..)) => {
                unreachable!("EnumVariantS instead of EnumVariant - compiler bug?")
            }
            (Self::Bool, Self::Bool)
            | (Self::Int, Self::Int)
            | (Self::Float, Self::Float)
            | (Self::String, Self::String) => true,
            (Self::Bool | Self::Int | Self::Float | Self::String, _) => false,
            (Self::Tuple(a), Self::Tuple(b)) => {
                if a.len() == b.len() {
                    a.iter()
                        .zip(b.iter())
                        .all(|(a, b)| a.fits_in(b, info).is_empty())
                } else {
                    false
                }
            }
            (Self::Tuple(_), _) => false,
            (Self::List(a), Self::List(b)) => a.fits_in(b, info).is_empty(),
            (Self::List(_), _) => false,
            (Self::Function(a), Self::Function(b)) => 'func_out: {
                for a in a {
                    'search: {
                        for b in b {
                            if a.1.fits_in(&b.1, info).is_empty()
                                && a.0.len() == b.0.len()
                                && a.0.iter().zip(b.0.iter()).all(|(a, b)| *a == *b)
                            {
                                break 'search;
                            }
                        }
                        break 'func_out false;
                    }
                }
                true
            }
            (Self::Function(..), _) => false,
            (Self::Thread(a), Self::Thread(b)) => a.fits_in(b, info).is_empty(),
            (Self::Thread(..), _) => false,
        };
        if info.log.vsingletype_fits_in.log() {
            info.log
                .log(LogMsg::VSingleTypeFitsIn(self.clone(), rhs.clone(), o));
        }
        o
    }
    pub fn fits_in_type(&self, rhs: &VType, info: &GlobalScriptInfo) -> bool {
        match self {
            Self::CustomType(t) => {
                rhs.types.iter().any(|rhs| {
                    if let Self::CustomType(rhs) = rhs {
                        *t == *rhs
                    } else {
                        false
                    }
                }) || info.custom_types[*t].fits_in(rhs, info).is_empty()
            }
            _ => rhs.types.iter().any(|b| self.fits_in(b, info)),
        }
    }
}

impl Into<VType> for VSingleType {
    fn into(self) -> VType {
        VType { types: vec![self] }
    }
}

//

pub struct VTypeWInfo(VType, GSInfo);
impl Display for VTypeWInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmtgs(
            f,
            Some(&self.1),
            &mut super::fmtgs::FormatInfo::default(),
            None,
        )
    }
}
impl VType {
    pub fn gsi(self, info: GSInfo) -> VTypeWInfo {
        VTypeWInfo(self, info)
    }
}

impl FormatGs for VSingleType {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),
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
            Self::List(v) => {
                write!(f, "[")?;
                v.fmtgs(f, info, form, file)?;
                write!(f, " ...]")
            }
            Self::Function(func) => {
                write!(f, "fn(")?;
                for (inputs, output) in func {
                    write!(f, "(")?;
                    for i in inputs {
                        i.fmtgs(f, info, form, file)?;
                        write!(f, " ");
                    }
                    output.fmtgs(f, info, form, file)?;
                    write!(f, ")")?;
                }
                write!(f, ")")
            }
            Self::Thread(out) => {
                write!(f, "thread(")?;
                out.fmtgs(f, info, form, file)?;
                write!(f, ")")
            }
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
                    write!(f, "{name}(")?;
                } else {
                    write!(f, "{variant}(")?;
                }
                inner.fmtgs(f, info, form, file)?;
                write!(f, ")")
            }
            Self::EnumVariantS(name, inner) => {
                write!(f, "{name}(")?;
                inner.fmtgs(f, info, form, file)?;
                write!(f, ")")
            }
            Self::CustomType(t) => {
                if let Some(info) = info {
                    #[cfg(not(debug_assertions))]
                    write!(
                        f,
                        "{}",
                        info.custom_type_names
                            .iter()
                            .find_map(|(name, id)| if *t == *id {
                                Some(name.to_owned())
                            } else {
                                None
                            })
                            .unwrap()
                    )?;
                    #[cfg(debug_assertions)]
                    write!(
                        f,
                        "{}/*{}*/",
                        info.custom_type_names
                            .iter()
                            .find_map(|(name, id)| if *t == *id {
                                Some(name.to_owned())
                            } else {
                                None
                            })
                            .unwrap(),
                        &info.custom_types[*t]
                    )?;
                    Ok(())
                } else {
                    write!(f, "[custom type #{t}]")
                }
            }
            Self::CustomTypeS(t) => write!(f, "{t}"),
        }
    }
}
impl Display for VSingleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None, &mut super::fmtgs::FormatInfo::default(), None)
    }
}

impl FormatGs for VType {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut super::fmtgs::FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result {
        for (i, t) in self.types.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            t.fmtgs(f, info, form, file)?;
        }
        Ok(())
    }
}
impl Display for VType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmtgs(f, None, &mut super::fmtgs::FormatInfo::default(), None)
    }
}
