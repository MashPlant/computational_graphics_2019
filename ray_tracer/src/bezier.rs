use super::vec::*;
use super::mesh::*;
use std::ops::*;
use crate::mat44::*;
use crate::geo::{HitResult, Ray};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BezierCurve {
  // the curve is originally on x-y plane
  // but we need to apply transform to it, so must use Vec3
  pub ps: Box<[F64Vec3]>,
  // derivative ps
  pub der_ps: Box<[F64Vec3]>,
}

impl BezierCurve {
  pub fn new(ps: Box<[F64Vec3]>) -> BezierCurve {
    let mut der_ps = std::iter::repeat(F64Vec3::zero()).take(ps.len() - 1).collect::<Box<[_]>>();
    let n = (ps.len() - 1) as f32;
    for (i, p) in der_ps.iter_mut().enumerate() {
      *p = (ps[i + 1] - ps[i]) * n;
    }
    BezierCurve { ps, der_ps }
  }

  // support both Vec2 & Vec3
  // the Vec3 version is used in BezierSurface
  fn value_impl<T>(ps: &mut [T], t: f32) -> T
    where T: Add<Output=T> + Mul<f32, Output=T> + Clone + Copy {
    for k in 1..ps.len() {
      for i in 0..ps.len() - k {
        ps[i] = ps[i] * (1.0 - t) + ps[i + 1] * t;
      }
    }
    ps[0]
  }

  pub fn value(&self, t: f32) -> Vec3 {
    BezierCurve::value_impl(&mut self.ps.clone(), t).to_f32_vec3()
  }

  pub fn tangent(&self, t: f32) -> Vec3 {
    BezierCurve::value_impl(&mut self.der_ps.clone(), t).to_f32_vec3()
  }

  pub fn rotate_bezier(mut self, sample_t: usize, sample_theta: usize, shift: Vec3, scale: f32) -> RotateBezier {
    for p in self.ps.as_mut() {
      *p *= scale;
      p.1 += shift.1 as f64;
    }
    for p in self.der_ps.as_mut() {
      *p *= scale;
    }
    let mesh = self.to_mesh(sample_t, sample_theta, Mat44::shift(shift.0, 0.0, shift.2));
    RotateBezier { curve: self, shift_x: shift.0, shift_z: shift.2, mesh: Box::new(mesh) }
  }
}

impl Curve for BezierCurve {
  // component z is ignored
  fn eval(&self, t: f32) -> (Vec2, Vec2) {
    (self.value(t).to_vec2(), self.tangent(t).to_vec2())
  }
}

#[derive(Serialize, Deserialize)]
pub struct RotateBezier {
  pub curve: BezierCurve,
  pub shift_x: f32,
  pub shift_z: f32,
  pub mesh: Box<Mesh>,
}

impl RotateBezier {
  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    // solving equation is not implemented in rust, but implemented in C++
    self.mesh.hit(ray)
  }
}

pub struct BezierSurface {
  ps: Box<[Vec3]>,
  width: usize,
}

impl BezierSurface {
  pub fn new(ps: Box<[Vec3]>, width: usize) -> BezierSurface {
    BezierSurface { ps, width }
  }

  pub fn value(&self, u: f32, v: f32) -> Vec3 {
    let (width, height) = (self.width, self.ps.len() / self.width);
    let mut ps = self.ps.clone();
    let ps = &mut ps;
    macro_rules! at {
      ($x: expr, $y: expr) => { ps[$y * width + $x] };
    }
    // get a bezier curve
    for k in 1..width {
      for j in 0..height {
        for i in 0..width - k {
          at!(i, j) = at!(i, j) * (1.0 - u) + at!(i + 1, j) * u;
        }
      }
    }
    // get a point from bezier curve
    for l in 1..height {
      for j in 0..height - l {
        at!(0, j) = at!(0, j) * (1.0 - v) + at!(0, j + 1) * v;
      }
    }
    at!(0, 0)
  }
}

impl Surface for BezierSurface {
  fn eval(&self, u: f32, v: f32) -> (Vec3, Vec3) {
    let (width, height) = (self.width, self.ps.len() / self.width);
    let mut ps = self.ps.clone();
    let ps = &mut ps;
    macro_rules! at {
      ($x: expr, $y: expr) => { ps[$y * width + $x] };
    }
    let (p, d_dv) = {
      for k in 1..width {
        for j in 0..height {
          for i in 0..width - k {
            at!(i, j) = at!(i, j) * (1.0 - u) + at!(i + 1, j) * u;
          }
        }
      }
      let mut der_ps = Vec::with_capacity(height - 1);
      for i in 0..height - 1 {
        der_ps.push((at!(0, i + 1) - at!(0, i)) * (height - 1) as f32);
      }
      let d_dv = BezierCurve::value_impl(&mut der_ps, v);
      for j in 1..height {
        for i in 0..height - j {
          at!(0, i) = at!(0, i) * (1.0 - v) + at!(0, i + 1) * v;
        }
      }
      (at!(0, 0), d_dv)
    };
    let d_du = {
      ps.copy_from_slice(&self.ps);
      for k in 1..height {
        for j in 0..width {
          for i in 0..height - k {
            at!(j, i) = at!(j, i) * (1.0 - u) + at!(j, i + 1) * u;
          }
        }
      }
      let mut der_ps = Vec::with_capacity(width - 1);
      for i in 0..width - 1 {
        der_ps.push((at!(i + 1, 0) - at!(i, 0)) * (width - 1) as f32);
      }
      BezierCurve::value_impl(&mut der_ps, u)
    };
    (p, d_du.cross(d_dv).norm())
  }
}