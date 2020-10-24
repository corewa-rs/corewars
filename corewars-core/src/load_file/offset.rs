use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A non-negative offset from the beginning of a core.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Offset {
    value: u32,
    core_size: u32,
}

impl Offset {
    /// Create a new Offset. The value will be adjusted to be within bounds of the core.
    ///
    /// Panics if `core_size` is invalid. Both 0 and `u32::MAX` are disallowed.
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
        result.set_value(value);
        result
    }

    /// Get the value of the offset. This will always be less than the core size.
    pub fn value(&self) -> u32 {
        self.value
    }

    /// Set the value of the offset. The value will be adjusted to be within
    /// bounds of the core size.
    pub fn set_value(&mut self, value: i32) {
        let core_isize = self.core_size as i32;
        let new_value = value.rem_euclid(core_isize);
        self.value = if new_value.is_negative() {
            new_value + core_isize
        } else {
            new_value
        } as u32;
    }

    /// Verify another offset has the same core size. Panics otherwise
    fn check_core_size(&self, other: &Self) {
        if self.core_size != other.core_size {
            panic!(
                "attempt to add mismatching core sizes: {} != {}",
                self.core_size, other.core_size
            )
        }
    }
}

/// Implement a `std::ops` operation for `Offset`.
macro_rules! impl_offset_op {
    ($op_trait:ident :: $op:ident , $assign_trait:ident :: $assign:ident ) => {
        impl $op_trait for Offset {
            type Output = Self;

            // Note $-expansion doesn't happen in doc comments. If needed there
            // is a workaround in https://github.com/rust-lang/rust/issues/52607

            /// Panics if the  right-hand side has a different `core_size`
            /// than the left-hand side.
            fn $op(self, rhs: Self) -> Self {
                self.check_core_size(&rhs);
                let mut result = Self::new(0, self.core_size);
                result.set_value((self.value as i32).$op(rhs.value as i32));
                result
            }
        }

        impl $assign_trait for Offset {
            fn $assign(&mut self, rhs: Self) {
                // check_core_size is called by $op_trait::$op
                *self = self.$op(rhs)
            }
        }
    };
}

impl_offset_op! { Add::add, AddAssign::add_assign }
impl_offset_op! { Sub::sub, SubAssign::sub_assign }
impl_offset_op! { Mul::mul, MulAssign::mul_assign }
impl_offset_op! { Div::div, DivAssign::div_assign }

/// Implement a `std::ops` operation for `Offset` and another type
macro_rules! impl_op {
    ($rhs:ty, $op_trait:ident :: $op:ident , $assign_trait:ident :: $assign:ident ) => {
        impl $op_trait<$rhs> for Offset {
            type Output = Self;

            fn $op(self, rhs: $rhs) -> Self::Output {
                self.$op(Self::new(rhs as i32, self.core_size))
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
impl_op! { i32, Mul::mul, MulAssign::mul_assign }
impl_op! { u32, Mul::mul, MulAssign::mul_assign }
impl_op! { i32, Div::div, DivAssign::div_assign }
impl_op! { u32, Div::div, DivAssign::div_assign }

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
    fn add_offset() {
        let mut offset = Offset::new(0, 12);

        assert_eq!(offset + 17i32, Offset::new(5, 12));
        assert_eq!(offset + -17i32, Offset::new(7, 12));
        assert_eq!(offset + Offset::new(17, 12), Offset::new(5, 12));
        assert_eq!(offset + Offset::new(-17, 12), Offset::new(7, 12));
        assert_eq!(offset + 17u32, Offset::new(5, 12));

        offset += 17i32;
        assert_eq!(offset, Offset::new(5, 12));
        offset = Offset::new(0, 12);

        offset += -17i32;
        assert_eq!(offset, Offset::new(7, 12));
        offset = Offset::new(0, 12);

        offset += Offset::new(17, 12);
        assert_eq!(offset, Offset::new(5, 12));
        offset = Offset::new(0, 12);

        offset += Offset::new(-17, 12);
        assert_eq!(offset, Offset::new(7, 12));
        offset = Offset::new(0, 12);

        offset += 17u32;
        assert_eq!(offset, Offset::new(5, 12));
    }

    #[test]
    fn sub_offset() {
        let mut offset = Offset::new(0, 12);

        assert_eq!(offset - 17i32, Offset::new(7, 12));
        assert_eq!(offset - -17i32, Offset::new(5, 12));
        assert_eq!(offset - Offset::new(17, 12), Offset::new(7, 12));
        assert_eq!(offset - Offset::new(-17, 12), Offset::new(5, 12));
        assert_eq!(offset - 17u32, Offset::new(7, 12));

        offset -= 17i32;
        assert_eq!(offset, Offset::new(7, 12));

        offset = Offset::new(0, 12);
        offset -= -17i32;
        assert_eq!(offset, Offset::new(5, 12));

        offset = Offset::new(0, 12);
        offset -= Offset::new(17, 12);
        assert_eq!(offset, Offset::new(7, 12));

        offset = Offset::new(0, 12);
        offset -= Offset::new(-17, 12);
        assert_eq!(offset, Offset::new(5, 12));

        offset = Offset::new(0, 12);
        offset -= 17u32;
        assert_eq!(offset, Offset::new(7, 12));
    }

    #[test]
    fn mul_offset() {
        let mut offset = Offset::new(2, 12);

        assert_eq!(offset * 5i32, Offset::new(10, 12));
        assert_eq!(offset * -5i32, Offset::new(2, 12));
        assert_eq!(offset * Offset::new(5, 12), Offset::new(10, 12));
        assert_eq!(offset * Offset::new(-5, 12), Offset::new(2, 12));
        assert_eq!(offset * 5u32, Offset::new(10, 12));

        offset *= 5i32;
        assert_eq!(offset, Offset::new(10, 12));

        offset = Offset::new(2, 12);
        offset *= -5i32;
        assert_eq!(offset, Offset::new(2, 12));

        offset = Offset::new(2, 12);
        offset *= Offset::new(5, 12);
        assert_eq!(offset, Offset::new(10, 12));

        offset = Offset::new(2, 12);
        offset *= Offset::new(-5, 12);
        assert_eq!(offset, Offset::new(2, 12));

        offset = Offset::new(2, 12);
        offset *= 5u32;
        assert_eq!(offset, Offset::new(10, 12));
    }

    #[test]
    fn div_offset() {
        let mut offset = Offset::new(10, 12);

        assert_eq!(offset / 5i32, Offset::new(2, 12));
        assert_eq!(offset / -5i32, Offset::new(1, 12));
        assert_eq!(offset / Offset::new(5, 12), Offset::new(2, 12));
        assert_eq!(offset / Offset::new(-5, 12), Offset::new(1, 12));
        assert_eq!(offset / 5u32, Offset::new(2, 12));

        offset /= 5i32;
        assert_eq!(offset, Offset::new(2, 12));

        offset = Offset::new(10, 12);
        offset /= -5i32;
        assert_eq!(offset, Offset::new(1, 12));

        offset = Offset::new(10, 12);
        offset /= Offset::new(5, 12);
        assert_eq!(offset, Offset::new(2, 12));

        offset = Offset::new(10, 12);
        offset /= Offset::new(-5, 12);
        assert_eq!(offset, Offset::new(1, 12));

        offset = Offset::new(10, 12);
        offset /= 5u32;
        assert_eq!(offset, Offset::new(2, 12));
    }
}
