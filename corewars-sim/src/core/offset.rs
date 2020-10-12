use std::ops::{Add, AddAssign, Sub, SubAssign};

/// An offset from the beginning of a core
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Offset {
    value: u32,
    core_size: u32,
}

impl Offset {
    /// Create a new Offset. Panics if `core_size` is invalid. Both 0 and
    /// [`u32::MAX`](u32::MAX) are disallowed.
    pub fn new<T, U>(value: T, core_size: U) -> Self
    where
        T: Into<i32>,
        U: Into<u32>,
    {
        let core_size = core_size.into();

        let core_isize = core_size as i32;
        // TODO: should there be a minimum allowed core size?
        if !core_isize.is_positive() {
            panic!(
                "Attempt to create offset with invalid core_size {}",
                core_isize
            )
        }

        let value = value.into().rem_euclid(core_isize);
        Self {
            value: value as u32,
            core_size,
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn set_value(&mut self, value: u32) {
        self.value = value.rem_euclid(self.core_size)
    }

    pub fn core_size(&self) -> u32 {
        self.core_size
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

        self.set_value(self.value + rhs.value)
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
                self.set_value((self.$op(rhs)).value)
            }
        }
    };
}

impl_op! { i32, Add::add, AddAssign::add_assign }
impl_op! { u32, Add::add, AddAssign::add_assign }
impl_op! { i32, Sub::sub, SubAssign::sub_assign }
impl_op! { u32, Sub::sub, SubAssign::sub_assign }
