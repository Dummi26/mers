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

#[derive(Debug, Clone, Copy)]
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
    fn subtypes(&self, acc: &mut Type) {
        // INT_MIN .. INT32U_MIN .. INT32S_MIN .. -128 .. -1 .. 0 .. 1 .. 127 .. 255 .. 65535 .. INT32S_MAX .. INT32U_MAX .. INT_MAX
        let mut add_range = |min, max| {
            // the range is non-empty, self starts before or where the range ends, and self ends after or where the range starts.
            if min <= max && self.0 <= max && min <= self.1 {
                acc.add(Arc::new(IntT(self.0.max(min), self.1.min(max))));
            }
        };
        add_range(INT_MIN, INT32U_MIN.saturating_sub(1));
        add_range(INT32U_MIN, INT32S_MIN.saturating_sub(1));
        add_range(INT32S_MIN, -129);
        add_range(-128, -2);
        add_range(-1, -1);
        add_range(0, 0);
        add_range(1, 1);
        add_range(2, 127);
        add_range(128, 255);
        add_range(256, 65535);
        add_range(65536, INT32S_MAX);
        add_range(INT32S_MAX.saturating_add(1), INT32U_MAX);
        add_range(INT32U_MAX.saturating_add(1), INT_MAX);
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
