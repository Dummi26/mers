use std::{fmt::Debug, ops::BitOr};

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

impl VSingleType {
    pub fn get_any(&self) -> Option<VType> {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::Function(..) | Self::Thread(..) => None,
            Self::String => Some(VSingleType::String.into()),
            Self::Tuple(t) => Some(t.iter().fold(VType { types: vec![] }, |a, b| a | b)),
            Self::List(t) => Some(t.clone()),
            Self::Reference(r) => r.get_any(),
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
    pub fn contains(&self, t: &VSingleType) -> bool {
        self.types.contains(t)
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
