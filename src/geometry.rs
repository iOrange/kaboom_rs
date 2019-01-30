#[macro_use]
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Copy, Clone)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3f { x, y, z }
    }

    pub fn norm(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&mut self) -> Self {
        *self = (*self) * (1.0 / self.norm());
        *self
    }
}

#[macro_export]
macro_rules! vec3 {
    ($($input: expr),*) => {
        Vec3f::new($( $input, )*)
    }
}

impl Mul for Vec3f {
    type Output = f32;

    fn mul(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Add for Vec3f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        vec3!(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        vec3!(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        vec3!(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Neg for Vec3f {
    type Output = Self;

    fn neg(self) -> Self {
        vec3!(-self.x, -self.y, -self.z)
    }
}
