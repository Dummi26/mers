use std::{collections::HashMap, fmt::Debug, ops::BitOr};

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
            Self::EnumVariant(_, t) | Self::EnumVariantS(_, t) => t.get(i),
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
    // returns Some(true) or Some(false) if all types are references or not references. If it is mixed or types is empty, returns None.
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
}

impl VSingleType {
    pub fn get_any(&self) -> Option<VType> {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::Function(..) | Self::Thread(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => Some(t.iter().fold(VType { types: vec![] }, |a, b| a | b)),
            Self::List(t) => Some(t.clone()),
            Self::Reference(r) => r.get_any(),
            Self::EnumVariant(_, t) => t.get_any(),
            Self::EnumVariantS(..) => unreachable!(),
        }
    }
    pub fn is_reference(&self) -> bool {
        match self {
            Self::Reference(_) => true,
            _ => false,
        }
    }
}
impl VType {
    pub fn get_any(&self) -> Option<VType> {
        let mut out = VType { types: vec![] };
        for t in &self.types {
            out = out | t.get_any()?; // if we can't use *get* on one type, we can't use it at all.
        }
        Some(out)
    }
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
    pub fn contains(&self, t: &VSingleType) -> bool {
        self.types.contains(t)
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
                        Some(out.clone())
                    } else {
                        None
                    }
                }) {
                    out.types
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
    pub fn noenum(self) -> VType {
        match self {
            Self::EnumVariant(_, v) | Self::EnumVariantS(_, v) => v,
            v => v.to(),
        }
    }
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
        }
    }
    pub fn fits_in(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Reference(r), Self::Reference(b)) => r.fits_in(b),
            (Self::Reference(_), _) | (_, Self::Reference(_)) => false,
            (Self::EnumVariant(v1, t1), Self::EnumVariant(v2, t2)) => {
                *v1 == *v2 && t1.fits_in(&t2).is_empty()
            }
            (Self::EnumVariant(..), _) | (_, Self::EnumVariant(..)) => false,
            (Self::EnumVariantS(..), _) | (_, Self::EnumVariantS(..)) => unreachable!(),
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
            (Self::Function(a), Self::Function(b)) => 'func_out: {
                for a in a {
                    'search: {
                        for b in b {
                            if a.1.fits_in(&b.1).is_empty()
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
            (Self::Thread(a), Self::Thread(b)) => a.fits_in(b).is_empty(),
            (Self::Thread(..), _) => false,
        }
    }
}

impl Into<VType> for VSingleType {
    fn into(self) -> VType {
        VType { types: vec![self] }
    }
}
