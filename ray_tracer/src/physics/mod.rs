// ONLY FOR RUN!!!
// make spheres moving in the scene, and draw the pictures all with gpu
#![allow(non_snake_case)]
#![allow(dead_code)]
use crate::geo::*;
use crate::vec::*;
use std::ops::*;

#[derive(Copy, Clone)]
pub struct MovingSphere {
  pub s: Sphere,
  pub v: Vec3,
  pub m: f32,
}

impl Deref for MovingSphere {
  type Target = Sphere;

  fn deref(&self) -> &<Self as Deref>::Target {
    &self.s
  }
}

impl DerefMut for MovingSphere {
  fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
    &mut self.s
  }
}

impl MovingSphere {
  fn do_collision(&mut self, other: &mut MovingSphere) {
//    let before_e = self.m * self.v.len2() + other.m * other.v.len2();
//    let before_p = self.v * self.m + other.v * other.m;

    let dir = self.c - other.c;
    let v = other.v - self.v;
    let vo = dir * dir.dot(v) / dir.len2();
    let vo1 = vo * (2.0 * other.m) / (self.m + other.m);
    let vo2 = vo * (other.m - self.m) / (self.m + other.m);
    self.v += vo1;
    other.v += vo2 - vo;

//    let after_e = self.m * self.v.len2() + other.m * other.v.len2();
//    let after_p = self.v * self.m + other.v * other.m;
//
//    assert!((after_e - before_e).abs() < 1e-1);
//    assert!((after_p - before_p).len2() < 1e-1);
  }

  fn collision(&mut self, other: &mut MovingSphere) {
    if (other.c - self.c).len2() < (self.r + other.r) * (self.r + other.r) {
      self.do_collision(other);
    }
  }

  fn gravity(&mut self, other: &mut MovingSphere, G: f32, step: f32) {
    let dir = self.c - other.c;
    let r2 = dir.len2();
    let r3 = r2 * r2.sqrt();
    let common = dir * G * step / r3;
    self.v += -common * other.m;
    other.v += common * self.m;
  }
}


#[derive(Default)]
pub struct PhyEmulator {
  pub ss: Vec<MovingSphere>,
  bound: ((f32, f32), (f32, f32), (f32, f32)),
  g: Vec3,
  G: f32,
  step: f32,
}

impl PhyEmulator {
  pub fn new(bound: ((f32, f32), (f32, f32), (f32, f32)), g: Vec3, G: f32, step: f32) -> PhyEmulator {
    PhyEmulator { ss: Vec::new(), bound, g, G, step }
  }

  pub fn next(&mut self) {
    for i in 0..self.ss.len() {
      let mut s = self.ss[i]; // copy here to pass borrow check
      for j in i + 1..self.ss.len() {
        s.gravity(&mut self.ss[j], self.G, self.step);
        s.collision(&mut self.ss[j]);
      }
      s.v += self.g * self.step;
      if s.c.0 - s.r <= (self.bound.0).0 || s.c.0 + s.r >= (self.bound.0).1 { s.v.0 = -s.v.0; }
      if s.c.1 - s.r <= (self.bound.1).0 || s.c.1 + s.r >= (self.bound.1).1 { s.v.1 = -s.v.1; }
      if s.c.2 - s.r <= (self.bound.2).0 || s.c.2 + s.r >= (self.bound.2).1 { s.v.2 = -s.v.2; }
      self.ss[i] = s;
    }
    for s in &mut self.ss {
      let v = s.v;
      s.c += v * self.step;
    }
  }
}