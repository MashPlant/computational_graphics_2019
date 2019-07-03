use super::vec::*;
use std::ops::Mul;
use std::default::Default;

// actually it is not performance bottle neck at all
// I write code like this just for fun
#[derive(Default, Copy, Clone)]
pub struct Mat44 {
  m00: f32,
  m01: f32,
  m02: f32,
  m03: f32,
  m10: f32,
  m11: f32,
  m12: f32,
  m13: f32,
  m20: f32,
  m21: f32,
  m22: f32,
  m23: f32,
  m30: f32,
  m31: f32,
  m32: f32,
  m33: f32,
}

// if you want to apply a sequence of operation
// use Mn * Mn-1 * ... * M1 * v
// then transform is performed from M1 to Mn
impl Mat44 {
  pub fn identity() -> Mat44 {
    Mat44 { m00: 1.0, m11: 1.0, m22: 1.0, ..Default::default() }
  }

  pub fn scale(x: f32, y: f32, z: f32) -> Mat44 {
    Mat44 { m00: x, m11: y, m22: z, m33: 1.0, ..Default::default() }
  }

  // use mat44 * vec3.extend(1.0) to do the shift
  // use mat44 * vec3.extend(0.0) can avoid the shift
  pub fn shift(x: f32, y: f32, z: f32) -> Mat44 {
    Mat44 { m00: 1.0, m11: 1.0, m22: 1.0, m33: 1.0, m03: x, m13: y, m23: z, ..Default::default() }
  }

  // rot_x means rotate AROUND x-axis, rot_y & rot_z are like it
  // counterclockwise, rot_y & rot_z are the same
  pub fn rot_x(theta: f32) -> Mat44 {
    let (sin, cos) = (theta.sin(), theta.cos());
    Mat44 { m00: 1.0, m11: cos, m12: -sin, m21: sin, m22: cos, m33: 1.0, ..Default::default() }
  }

  pub fn rot_x_deg(deg: f32) -> Mat44 {
    Mat44::rot_x(deg.to_radians())
  }

  pub fn rot_y(theta: f32) -> Mat44 {
    let (sin, cos) = (theta.sin(), theta.cos());
    Mat44 { m00: cos, m02: sin, m11: 1.0, m20: -sin, m22: cos, m33: 1.0, ..Default::default() }
  }

  pub fn rot_y_deg(deg: f32) -> Mat44 {
    Mat44::rot_y(deg.to_radians())
  }

  pub fn rot_z(theta: f32) -> Mat44 {
    let (sin, cos) = (theta.sin(), theta.cos());
    Mat44 { m00: cos, m01: -sin, m10: sin, m11: cos, m22: 1.0, m33: 1.0, ..Default::default() }
  }

  pub fn rot_z_deg(deg: f32) -> Mat44 {
    Mat44::rot_z(deg.to_radians())
  }
}

impl Mul<Mat44> for Mat44 {
  type Output = Mat44;

  fn mul(self, rhs: Mat44) -> Mat44 {
    Mat44 {
      m00: rhs.m00 * self.m00 + rhs.m10 * self.m01 + rhs.m20 * self.m02 + rhs.m30 * self.m03,
      m01: rhs.m01 * self.m00 + rhs.m11 * self.m01 + rhs.m21 * self.m02 + rhs.m31 * self.m03,
      m02: rhs.m02 * self.m00 + rhs.m12 * self.m01 + rhs.m22 * self.m02 + rhs.m32 * self.m03,
      m03: rhs.m03 * self.m00 + rhs.m13 * self.m01 + rhs.m23 * self.m02 + rhs.m33 * self.m03,
      m10: rhs.m00 * self.m10 + rhs.m10 * self.m11 + rhs.m20 * self.m12 + rhs.m30 * self.m13,
      m11: rhs.m01 * self.m10 + rhs.m11 * self.m11 + rhs.m21 * self.m12 + rhs.m31 * self.m13,
      m12: rhs.m02 * self.m10 + rhs.m12 * self.m11 + rhs.m22 * self.m12 + rhs.m32 * self.m13,
      m13: rhs.m03 * self.m10 + rhs.m13 * self.m11 + rhs.m23 * self.m12 + rhs.m33 * self.m13,
      m20: rhs.m00 * self.m20 + rhs.m10 * self.m21 + rhs.m20 * self.m22 + rhs.m30 * self.m23,
      m21: rhs.m01 * self.m20 + rhs.m11 * self.m21 + rhs.m21 * self.m22 + rhs.m31 * self.m23,
      m22: rhs.m02 * self.m20 + rhs.m12 * self.m21 + rhs.m22 * self.m22 + rhs.m32 * self.m23,
      m23: rhs.m03 * self.m20 + rhs.m13 * self.m21 + rhs.m23 * self.m22 + rhs.m33 * self.m23,
      m30: rhs.m00 * self.m30 + rhs.m10 * self.m31 + rhs.m20 * self.m32 + rhs.m30 * self.m33,
      m31: rhs.m01 * self.m30 + rhs.m11 * self.m31 + rhs.m21 * self.m32 + rhs.m31 * self.m33,
      m32: rhs.m02 * self.m30 + rhs.m12 * self.m31 + rhs.m22 * self.m32 + rhs.m32 * self.m33,
      m33: rhs.m03 * self.m30 + rhs.m13 * self.m31 + rhs.m23 * self.m32 + rhs.m33 * self.m33,
    }
  }
}

impl Mul<Vec4> for Mat44 {
  type Output = Vec4;

  fn mul(self, rhs: Vec4) -> Vec4 {
    Vec4(
      rhs.0 * self.m00 + rhs.1 * self.m01 + rhs.2 * self.m02 + rhs.3 * self.m03,
      rhs.0 * self.m10 + rhs.1 * self.m11 + rhs.2 * self.m12 + rhs.3 * self.m13,
      rhs.0 * self.m20 + rhs.1 * self.m21 + rhs.2 * self.m22 + rhs.3 * self.m23,
      rhs.0 * self.m30 + rhs.1 * self.m31 + rhs.2 * self.m32 + rhs.3 * self.m33,
    )
  }
}