use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use image::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vector3(pub f64, pub f64, pub f64);

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs;
        self.1 /= rhs;
        self.2 /= rhs;
    }
}

impl Mul for Vector3 {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
}

impl Div for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: Self) -> Self::Output {
        Self(
            self.1 * rhs.2 - self.2 * rhs.1,
            self.2 * rhs.0 - self.0 * rhs.2,
            self.0 * rhs.1 - self.1 * rhs.0,
        )
    }
}

impl Vector3 {
    pub fn len(self) -> f64 {
        (self * self).sqrt()
    }

    pub fn normalize(self) -> Self {
        self / self.len()
    }

    pub fn is_small(self) -> bool {
        self.0.abs() < 1e-8 || self.1.abs() < 1e-8 || self.2.abs() < 1e-8
    }

    pub fn attune(self, other: Vector3) -> Self {
        Self(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

fn f(x: f64) -> f64 {
    if x >= 0.0031308 {
        (1.055) * x.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * x
    }
}

impl From<Vector3> for Rgb<u8> {
    fn from(value: Vector3) -> Rgb<u8> {
        Rgb([
            (f(value.0) * 255.0) as u8,
            (f(value.1) * 255.0) as u8,
            (f(value.2) * 255.0) as u8,
        ])
    }
}
