use crate::{vec_declare_constants, vec_implement_common_functions, vec_implement_common_traits, vec_implement_operators_overloading};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3 {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Vec3 {
  vec_implement_common_functions!(3, 0 => x: f64, 1 => y: f64, 2 => z: f64);

  #[inline]
  pub fn dot(self, other: Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }

  #[inline]
  pub fn cross(self, other: Self) -> Self {
    Self {
      x: self.y * other.z - other.y * self.z,
      y: self.z * other.x - other.z * self.x,
      z: self.x * other.y - other.x * self.y,
    }
  }

  #[inline]
  pub fn length_squared(self) -> f64 { self.dot(self) }

  #[inline]
  pub fn length(self) -> f64 { self.length_squared().sqrt() }

  #[inline]
  pub fn normalize(self) -> Self { self / self.length() }

  vec_declare_constants!(Vec3);
}

vec_implement_operators_overloading!(Vec3, x, y, z);
vec_implement_common_traits!(Vec3, 3, 0 => x: f64, 1 => y: f64, 2 => z: f64);

