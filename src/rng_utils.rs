use std::{cell::UnsafeCell, f64, rc::Rc};

use rand::{Rng, RngCore, SeedableRng, distr::Distribution};
use rand_xoshiro::Xoshiro128StarStar;

use crate::Vector3;

#[derive(Clone, Debug)]
pub struct XoshiroThreadRng {
    rng: Rc<UnsafeCell<Xoshiro128StarStar>>,
}

thread_local!(
    static THREAD_RNG_KEY: Rc<UnsafeCell<Xoshiro128StarStar>> =
        Rc::new(UnsafeCell::new(Xoshiro128StarStar::from_os_rng()));
);

pub fn rng() -> XoshiroThreadRng {
    let rng = THREAD_RNG_KEY.with(|t| t.clone());
    XoshiroThreadRng { rng }
}

impl RngCore for XoshiroThreadRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.next_u64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.fill_bytes(dest)
    }
}

pub struct UniformSphere;

impl Distribution<Vector3> for UniformSphere {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3 {
        let theta = rng.random::<f64>() * f64::consts::TAU;
        let z = (rng.random::<f64>() - 0.5) * 2.0;
        let r = 1.0 - z * z;
        Vector3(r * theta.cos(), r * theta.sin(), z)
    }
}

pub struct UniformDisc;

impl Distribution<(f64, f64)> for UniformDisc {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> (f64, f64) {
        let theta = rng.random::<f64>() * f64::consts::TAU;
        let r = rng.random::<f64>().sqrt();
        (theta.sin() * r, theta.cos() * r)
    }
}
