use std::{any::Any, fmt::Display, sync::Arc};

use crate::info::DisplayInfo;

use super::{MersData, MersType, Type};

/// The smallest representable integer.
/// Depends on the system for which mers is being compiled, as mers uses pointer-sized signed integers.
/// `-2^W`, `W` is the bit-width of a pointer on the system, often `32` or `64`, minus one.
pub const INT_MIN: isize = isize::MIN;
/// The largest representable integer.
/// Depends on the system for which mers is being compiled, as mers uses pointer-sized signed integers.
/// `2^W-1`, `W` is the bit-width of a pointer on the system, often `32` or `64`, minus one.
pub const INT_MAX: isize = isize::MAX;
/// The smallest integer representable by mers and by a signed 32-bit number.
/// `max(INT_MIN, -2^31)`
pub const INT32S_MIN: isize = if isize::BITS > i32::BITS {
    i32::MIN as isize
} else {
    isize::MIN
};
/// The largest integer representable by mers and by a signed 32-bit number.
/// `min(INT_MAX, 2^31-1)`
pub const INT32S_MAX: isize = if isize::BITS > i32::BITS {
    i32::MAX as isize
} else {
    isize::MAX
};
/// The smallest integer representable by mers and by an unsigned 32-bit number, assuming its value was negative.
/// `max(INT_MIN, -(2^32-1))`
pub const INT32U_MIN: isize = if isize::BITS > u32::BITS {
    -(u32::MAX as isize)
} else {
    isize::MIN
};
/// The largest integer representable by mers and by an unsigned 32-bit number.
/// `min(INT_MAX, 2^32-1)`
pub const INT32U_MAX: isize = if isize::BITS > u32::BITS {
    u32::MAX as isize
} else {
    isize::MAX
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Int(pub isize);

impl MersData for Int {
    fn display(&self, _info: &DisplayInfo<'_>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
    fn is_eq(&self, other: &dyn MersData) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            other.0 == self.0
        } else {
            false
        }
    }
    fn clone(&self) -> Box<dyn MersData> {
        Box::new(Clone::clone(self))
    }
    fn as_type(&self) -> super::Type {
        Type::new(IntT(self.0, self.0))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn to_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntT(pub isize, pub isize);
impl MersType for IntT {
    fn display(
        &self,
        _info: &crate::info::DisplayInfo<'_>,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(f, "{self}")
    }
    fn is_same_type_as(&self, other: &dyn MersType) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    fn is_included_in(&self, target: &dyn MersType) -> bool {
        if let Some(target) = target.as_any().downcast_ref::<Self>() {
            target.0 <= self.0 && self.1 <= target.1
        } else {
            false
        }
    }
    fn without(&self, remove: &dyn MersType) -> Option<Type> {
        if self.is_included_in(remove) {
            Some(Type::empty())
        } else if let Some(remove) = remove.as_any().downcast_ref::<Self>() {
            if remove.0 <= self.0 && self.1 <= remove.1 {
                Some(Type::empty())
            } else if remove.0 <= self.0 && self.0 <= remove.1 && remove.1 <= self.1 {
                if remove.1 + 1 <= self.1 {
                    Some(Type::new(Self(remove.1 + 1, self.1)))
                } else {
                    Some(Type::empty())
                }
            } else if self.0 <= remove.0 && remove.0 <= self.1 && self.1 <= remove.1 {
                if self.0 <= remove.0 + 1 {
                    Some(Type::new(Self(self.0, remove.0 - 1)))
                } else {
                    Some(Type::empty())
                }
            } else if self.0 < remove.0 && remove.0 <= remove.1 && remove.1 < self.1 {
                Some(Type::newm(vec![
                    Arc::new(Self(self.0, remove.0 - 1)),
                    Arc::new(Self(remove.1 + 1, self.1)),
                ]))
            } else {
                None
            }
        } else {
            None
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn to_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

impl Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Display for IntT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.0, self.1) {
            (isize::MIN, isize::MAX) => write!(f, "Int"),
            (val, val2) if val == val2 => write!(f, "Int<{val}>"),
            (min, isize::MAX) => write!(f, "Int<{min}..>"),
            (isize::MIN, max) => write!(f, "Int<..{max}>"),
            (min, max) => write!(f, "Int<{min}..{max}>"),
        }
    }
}
