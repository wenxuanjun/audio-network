use core::fmt;
use num_traits::{cast, NumCast};
use std::ops::{Add, AddAssign, Div, DivAssign};
use std::ops::{Mul, MulAssign, Neg, Sub, SubAssign};

#[cfg(feature = "fixed_point")]
use fixed::traits::{FromFixed, ToFixed};
#[cfg(feature = "fixed_point")]
use fixed::types::I32F32;

#[cfg(not(feature = "fixed_point"))]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FP(f32);

#[cfg(feature = "fixed_point")]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FP(I32F32);

#[cfg(not(feature = "fixed_point"))]
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

#[cfg(feature = "fixed_point")]
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

impl Neg for FP {
    type Output = Self;
    fn neg(self) -> FP {
        FP(-self.0)
    }
}
impl Add for FP {
    type Output = Self;
    fn add(self, rhs: FP) -> Self {
        FP(self.0 + rhs.0)
    }
}
impl Sub for FP {
    type Output = Self;
    fn sub(self, rhs: FP) -> Self {
        FP(self.0 - rhs.0)
    }
}
impl Mul for FP {
    type Output = Self;
    fn mul(self, rhs: FP) -> Self {
        FP(self.0 * rhs.0)
    }
}
impl Div for FP {
    type Output = Self;
    fn div(self, rhs: FP) -> Self {
        FP(self.0 / rhs.0)
    }
}
impl AddAssign for FP {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl SubAssign for FP {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl MulAssign for FP {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}
impl DivAssign for FP {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl std::iter::Sum for FP {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}
impl fmt::Display for FP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}
