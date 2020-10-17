use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A non-negative offset from the beginning of a core.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Offset {
    value: u32,
    core_size: u32,
}

impl Offset {
    /// Create a new Offset. The value will be adjusted to be within bounds of the core.
    ///
    /// Panics if `core_size` is invalid. Both 0 and [`u32::MAX`](u32::MAX) are disallowed.
    pub fn new(value: i32, core_size: u32) -> Self {
        // TODO: should there be a minimum allowed core size?
        let core_isize = core_size as i32;
        if !core_isize.is_positive() {
            panic!(
                "Attempt to create offset with invalid core_size {}",
                core_isize
            )
        }

        let mut result = Self {
            value: 0,
            core_size,
        };
        result.set_value(value.rem_euclid(core_isize));
        result
    }

    /// Get the value of the offset. This will always be less than the core size.
    pub fn value(&self) -> u32 {
        self.value
    }

    /// Set the value of the offset. The value will be adjusted to be within
    /// bounds of the core size.
    pub fn set_value(&mut self, value: i32) {
        self.value = value.rem_euclid(self.core_size as i32) as u32
    }
}

impl Add for Offset {
    type Output = Self;

    /// Add two offsets together. Panics if the right-hand side's has a different
    /// `core_size` than the left-hand side
    fn add(self, rhs: Self) -> Self::Output {
        if self.core_size != rhs.core_size {
            panic!(
                "attempt to add mismatching core sizes: {} != {}",
                self.core_size, rhs.core_size
            )
        }

        self + (rhs.value as i32)
    }
}

impl AddAssign<Offset> for Offset {
    /// Adds another offset to this one. Panics if the right-hand side's has a different
    /// `core_size` than the left-hand side
    fn add_assign(&mut self, rhs: Offset) {
        if self.core_size != rhs.core_size {
            panic!(
                "attempt to add mismatching core sizes: {} != {}",
                self.core_size, rhs.core_size
            )
        }

        *self = *self + rhs
    }
}

macro_rules! impl_op {
    ($rhs:ty, $op_trait:ident :: $op:ident , $assign_trait:ident :: $assign:ident ) => {
        impl $op_trait<$rhs> for Offset {
            type Output = Self;

            fn $op(self, rhs: $rhs) -> Self::Output {
                let value = (self.value as $rhs).$op(rhs) as i32;
                Self::new(value, self.core_size)
            }
        }

        impl $assign_trait<$rhs> for Offset {
            fn $assign(&mut self, rhs: $rhs) {
                self.set_value((self.$op(rhs)).value as _)
            }
        }
    };
}

impl_op! { i32, Add::add, AddAssign::add_assign }
impl_op! { u32, Add::add, AddAssign::add_assign }
impl_op! { i32, Sub::sub, SubAssign::sub_assign }
impl_op! { u32, Sub::sub, SubAssign::sub_assign }

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn create_offset() {
        let offset = Offset::new(1234, 12);
        assert_eq!(offset.value(), 10);
    }

    #[test]
    fn set_offset_value() {
        let mut offset = Offset::new(1234, 12);
        offset.set_value(20);
        assert_eq!(offset.value(), 8);
    }

    #[test]
    fn add_to_offset() {
        let mut offset = Offset::new(0, 12);

        assert_eq!(offset + 20i32, Offset::new(8, 12));
        assert_eq!(offset + 20u32, Offset::new(8, 12));

        offset += 20i32;
        assert_eq!(offset, Offset::new(8, 12));
        offset += 20u32;
        assert_eq!(offset, Offset::new(4, 12));
    }
}
