use std::ops::*;
use std::hash::*;
use std::mem;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl Vec3 {
  pub fn zero() -> Vec3 {
    Vec3(0.0, 0.0, 0.0)
  }

  pub fn dot(&self, rhs: Vec3) -> f32 {
    self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
  }

  pub fn cross(&self, rhs: Vec3) -> Vec3 {
    Vec3(self.1 * rhs.2 - self.2 * rhs.1, self.2 * rhs.0 - self.0 * rhs.2, self.0 * rhs.1 - self.1 * rhs.0)
  }

  // entry-wise product
  pub fn schur(&self, rhs: Vec3) -> Vec3 {
    Vec3(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
  }

  pub fn norm(&self) -> Vec3 {
    *self * (1.0 / self.len())
  }

  pub fn len2(&self) -> f32 {
    self.dot(*self)
  }

  pub fn len(&self) -> f32 {
    self.dot(*self).sqrt()
  }

  // self is a unit vec, return a unit vec that is orthogonal to self
  pub fn get_orthogonal_for_unit(&self) -> Vec3 {
    if self.1.abs() != 1.0 { Vec3(self.2, 0.0, -self.0) } else { Vec3(0.0, self.2, -self.1) }.norm()
  }

  pub fn extend(&self, w: f32) -> Vec4 {
    Vec4(self.0, self.1, self.2, w)
  }

  pub fn to_vec2(&self) -> Vec2 {
    Vec2(self.0, self.1)
  }
}

impl Index<usize> for Vec3 {
  type Output = f32;

  fn index(&self, index: usize) -> &f32 {
    unsafe { &*(self as *const Vec3 as *const f32).add(index) }
  }
}

impl IndexMut<usize> for Vec3 {
  fn index_mut(&mut self, index: usize) -> &mut f32 {
    unsafe { &mut *(self as *mut Vec3 as *mut f32).add(index) }
  }
}

// well, though language design says float is not Hash & Eq & Ord
// when I need them, this cannot stop me
impl Hash for Vec3 {
  fn hash<H: Hasher>(&self, state: &mut H) {
    unsafe {
      Hash::hash(&mem::transmute::<_, [u8; mem::size_of::<Vec3>()]>(*self), state)
    }
  }
}

impl Eq for Vec3 {}

impl Add for Vec3 {
  type Output = Vec3;

  fn add(self, rhs: Vec3) -> Vec3 {
    Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
  }
}

impl AddAssign for Vec3 {
  fn add_assign(&mut self, rhs: Vec3) {
    self.0 += rhs.0;
    self.1 += rhs.1;
    self.2 += rhs.2;
  }
}

impl Neg for Vec3 {
  type Output = Vec3;

  fn neg(self) -> Vec3 {
    Vec3(-self.0, -self.1, -self.2)
  }
}

impl Sub for Vec3 {
  type Output = Vec3;

  fn sub(self, rhs: Vec3) -> Vec3 {
    Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
  }
}

impl SubAssign for Vec3 {
  fn sub_assign(&mut self, rhs: Vec3) {
    self.0 -= rhs.0;
    self.1 -= rhs.1;
    self.2 -= rhs.2;
  }
}

impl Mul<f32> for Vec3 {
  type Output = Vec3;

  fn mul(self, rhs: f32) -> Vec3 {
    Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
  }
}

impl MulAssign<f32> for Vec3 {
  fn mul_assign(&mut self, rhs: f32) {
    self.0 *= rhs;
    self.1 *= rhs;
    self.2 *= rhs;
  }
}

impl Div<f32> for Vec3 {
  type Output = Vec3;

  fn div(self, rhs: f32) -> Vec3 {
    self * (1.0 / rhs)
  }
}

impl DivAssign<f32> for Vec3 {
  fn div_assign(&mut self, rhs: f32) {
    *self *= 1.0 / rhs;
  }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct F64Vec3(pub f64, pub f64, pub f64);

impl F64Vec3 {
  pub fn zero() -> F64Vec3 {
    F64Vec3(0.0, 0.0, 0.0)
  }

  pub fn dot(&self, rhs: F64Vec3) -> f64 {
    self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
  }

  pub fn cross(&self, rhs: F64Vec3) -> F64Vec3 {
    F64Vec3(self.1 * rhs.2 - self.2 * rhs.1, self.2 * rhs.0 - self.0 * rhs.2, self.0 * rhs.1 - self.1 * rhs.0)
  }

  // entry-wise product
  pub fn schur(&self, rhs: F64Vec3) -> F64Vec3 {
    F64Vec3(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
  }

  pub fn norm(&self) -> F64Vec3 {
    *self * (1.0 / self.len())
  }

  pub fn len2(&self) -> f64 {
    self.dot(*self)
  }

  pub fn len(&self) -> f64 {
    self.dot(*self).sqrt()
  }

  pub fn to_f32_vec3(&self) -> Vec3 {
    Vec3(self.0 as f32, self.1 as f32, self.2 as f32)
  }
}

impl Eq for F64Vec3 {}

impl Add for F64Vec3 {
  type Output = F64Vec3;

  fn add(self, rhs: F64Vec3) -> F64Vec3 {
    F64Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
  }
}

impl AddAssign for F64Vec3 {
  fn add_assign(&mut self, rhs: F64Vec3) {
    self.0 += rhs.0;
    self.1 += rhs.1;
    self.2 += rhs.2;
  }
}

impl Neg for F64Vec3 {
  type Output = F64Vec3;

  fn neg(self) -> F64Vec3 {
    F64Vec3(-self.0, -self.1, -self.2)
  }
}

impl Sub for F64Vec3 {
  type Output = F64Vec3;

  fn sub(self, rhs: F64Vec3) -> F64Vec3 {
    F64Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
  }
}

impl SubAssign for F64Vec3 {
  fn sub_assign(&mut self, rhs: F64Vec3) {
    self.0 -= rhs.0;
    self.1 -= rhs.1;
    self.2 -= rhs.2;
  }
}

impl Mul<f32> for F64Vec3 {
  type Output = F64Vec3;

  fn mul(self, rhs: f32) -> F64Vec3 {
    F64Vec3(self.0 * rhs as f64, self.1 * rhs as f64, self.2 * rhs as f64)
  }
}

impl Mul<f64> for F64Vec3 {
  type Output = F64Vec3;

  fn mul(self, rhs: f64) -> F64Vec3 {
    F64Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
  }
}

impl MulAssign<f32> for F64Vec3 {
  fn mul_assign(&mut self, rhs: f32) {
    self.0 *= rhs as f64;
    self.1 *= rhs as f64;
    self.2 *= rhs as f64;
  }
}

impl MulAssign<f64> for F64Vec3 {
  fn mul_assign(&mut self, rhs: f64) {
    self.0 *= rhs;
    self.1 *= rhs;
    self.2 *= rhs;
  }
}

impl Div<f32> for F64Vec3 {
  type Output = F64Vec3;

  fn div(self, rhs: f32) -> F64Vec3 {
    self * (1.0 / rhs)
  }
}

impl Div<f64> for F64Vec3 {
  type Output = F64Vec3;

  fn div(self, rhs: f64) -> F64Vec3 {
    self * (1.0 / rhs)
  }
}

impl DivAssign<f32> for F64Vec3 {
  fn div_assign(&mut self, rhs: f32) {
    *self *= 1.0 / rhs;
  }
}

impl DivAssign<f64> for F64Vec3 {
  fn div_assign(&mut self, rhs: f64) {
    *self *= 1.0 / rhs;
  }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Serialize, Deserialize)]
pub struct Vec2(pub f32, pub f32);

impl Vec2 {
  pub fn zero() -> Vec2 {
    Vec2(0.0, 0.0)
  }

  pub fn dot(&self, rhs: &Vec2) -> f32 {
    self.0 * rhs.0 + self.1 * rhs.1
  }

  pub fn len2(&self) -> f32 {
    self.dot(self)
  }

  pub fn len(&self) -> f32 {
    self.dot(self).sqrt()
  }
}

impl Hash for Vec2 {
  fn hash<H: Hasher>(&self, state: &mut H) {
    unsafe {
      Hash::hash(&mem::transmute::<_, [u8; mem::size_of::<Vec2>()]>(*self), state)
    }
  }
}

impl Eq for Vec2 {}

impl Add for Vec2 {
  type Output = Vec2;

  fn add(self, rhs: Vec2) -> Vec2 {
    Vec2(self.0 + rhs.0, self.1 + rhs.1)
  }
}

impl Sub for Vec2 {
  type Output = Vec2;

  fn sub(self, rhs: Vec2) -> Vec2 {
    Vec2(self.0 - rhs.0, self.1 - rhs.1)
  }
}

impl Mul<f32> for Vec2 {
  type Output = Vec2;

  fn mul(self, rhs: f32) -> Vec2 {
    Vec2(self.0 * rhs, self.1 * rhs)
  }
}

impl Div<f32> for Vec2 {
  type Output = Vec2;

  fn div(self, rhs: f32) -> Vec2 {
    self * (1.0 / rhs)
  }
}

pub struct Vec4(pub f32, pub f32, pub f32, pub f32);

impl Vec4 {
  pub fn zero() -> Vec4 {
    Vec4(0.0, 0.0, 0.0, 0.0)
  }

  pub fn to_vec3(&self) -> Vec3 {
    Vec3(self.0, self.1, self.2)
  }
}