use core::fmt;
use num_traits::{cast, NumCast};
use std::ops::{Add, AddAssign, Div, DivAssign};
use std::ops::{Mul, MulAssign, Neg, Sub, SubAssign};

cfg_if::cfg_if! {
    if #[cfg(feature = "fixed_point")] {
        use fixed::traits::{FromFixed, ToFixed};
        use fixed::types::I32F32;

        #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct FP(I32F32);

        impl FP {
            pub const ONE: Self = Self(I32F32::ONE);
            pub const ZERO: Self = Self(I32F32::ZERO);
            pub const PI: Self = Self(I32F32::PI);
        
            pub fn abs(self) -> Self {
                Self(self.0.abs())
            }
            pub fn sin(self) -> Self {
                Self(cordic::sin(self.0))
            }
        
            pub fn from<T: NumCast>(x: T) -> Self {
                Self(cast::<T, f32>(x).unwrap().to_fixed())
            }
            pub fn into<T: FromFixed>(self) -> T {
                T::from_fixed(self.0)
            }
        }
    } else {
        #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct FP(f32);

        impl FP {
            pub const ONE: Self = Self(1.0);
            pub const ZERO: Self = Self(0.0);
            pub const PI: Self = Self(std::f32::consts::PI);
        
            pub fn abs(self) -> Self {
                Self(self.0.abs())
            }
            pub fn sin(self) -> Self {
                Self(self.0.sin())
            }
        
            pub fn from<T: NumCast>(x: T) -> Self {
                Self(cast::<T, f32>(x).unwrap())
            }
            pub fn into<T: NumCast>(self) -> T {
                cast::<f32, T>(self.0).unwrap()
            }
        }
    }
}

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for FP {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self {
                FP(self.0 $op rhs.0)
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for FP {
            fn $method(&mut self, rhs: Self) {
                self.0 $op rhs.0;
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);

impl_op_assign!(AddAssign, add_assign, +=);
impl_op_assign!(SubAssign, sub_assign, -=);
impl_op_assign!(MulAssign, mul_assign, *=);
impl_op_assign!(DivAssign, div_assign, /=);

impl Neg for FP {
    type Output = Self;

    fn neg(self) -> Self {
        FP(-self.0)
    }
}

impl std::iter::Sum for FP {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl fmt::Display for FP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
