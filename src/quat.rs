use std::ops::{Mul, MulAssign};

use crate::Vector3;

fn square(x: f64) -> f64 {
    x * x
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Quaternion(pub f64, pub f64, pub f64, pub f64);

impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        let w = self.0 * rhs.0 - self.1 * rhs.1 - self.2 * rhs.2 - self.3 * rhs.3;
        let x = self.0 * rhs.1 + self.1 * rhs.0 + self.2 * rhs.3 - self.3 * rhs.2;
        let y = self.0 * rhs.2 - self.1 * rhs.3 + self.2 * rhs.0 + self.3 * rhs.1;
        let z = self.0 * rhs.3 + self.1 * rhs.2 - self.2 * rhs.1 + self.3 * rhs.0;
        Self(w, x, y, z)
    }
}

impl MulAssign for Quaternion {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<[f64; 4]> for Quaternion {
    fn from(value: [f64; 4]) -> Self {
        Self(value[0], value[1], value[2], value[3])
    }
}

impl Quaternion {
    pub fn norm(&self) -> f64 {
        f64::sqrt(square(self.0) + square(self.1) + square(self.2) + square(self.3))
    }

    pub fn normalize(&mut self) {
        let n = self.norm();
        if n < f64::EPSILON {
            return;
        }
        self.0 /= n;
        self.1 /= n;
        self.2 /= n;
        self.3 /= n;
    }

    pub fn conj(self) -> Self {
        Self(self.0, -self.1, -self.2, -self.3)
    }

    pub fn from_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        let cr = f64::cos(roll * 0.5);
        let sr = f64::sin(roll * 0.5);
        let cp = f64::cos(pitch * 0.5);
        let sp = f64::sin(pitch * 0.5);
        let cy = f64::cos(yaw * 0.5);
        let sy = f64::sin(yaw * 0.5);

        let w = cp * cy * cr + sp * sy * sr;
        let x = sp * cy * cr - cp * sy * sr;
        let y = cp * sy * cr + sp * cy * sr;
        let z = cp * cy * sr - sp * sy * cr;

        Self(w, x, y, z)
    }

    pub fn rotate(&self, v: Vector3) -> Vector3 {
        let x = (1.0 - 2.0 * self.2 * self.2 - 2.0 * self.3 * self.3) * v.0
            + 2.0 * v.1 * (self.2 * self.1 - self.0 * self.3)
            + 2.0 * v.2 * (self.0 * self.2 + self.3 * self.1);
        let y = 2.0 * v.0 * (self.0 * self.3 + self.2 * self.1)
            + v.1 * (1.0 - 2.0 * self.1 * self.1 - 2.0 * self.3 * self.3)
            + 2.0 * v.2 * (self.2 * self.3 - self.1 * self.0);
        let z = 2.0 * v.0 * (self.3 * self.1 - self.0 * self.2)
            + 2.0 * v.1 * (self.0 * self.1 + self.3 * self.2)
            + v.2 * (1.0 - 2.0 * self.1 * self.1 - 2.0 * self.2 * self.2);
        Vector3(x, y, z)
    }
}
