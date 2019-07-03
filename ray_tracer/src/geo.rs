use super::vec::*;
use super::material::*;
use super::util::*;
use super::mesh::Mesh;
use serde::{Serialize, Deserialize};
use std::f32::consts::PI;
use crate::bezier::RotateBezier;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Ray {
  pub o: Vec3,
  pub d: Vec3, // d is normalized
}

impl Ray {
  // if you are confident that d is already normalized, needn't to call Ray::new
  pub fn new(o: Vec3, d: Vec3) -> Ray {
    Ray { o, d: d.norm() }
  }
}

pub struct HitResult {
  pub t: f32,
  pub norm: Vec3,
  pub uv: Vec2,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Sphere {
  pub c: Vec3,
  pub r: f32,
}

impl Sphere {
  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    // let: d=ray.d, o=ray.o, c=sphere.c, r=sphere.r
    // solve: dot(d,d) * t^2 + dot(2d, o-c) * t + dot(o-c,o-c)-r^2 = 0
    // note that dot(d,d)=d.len2()=1
    let oc = self.c - ray.o;
    let b = oc.dot(ray.d);
    let det = b * b - oc.len2() + self.r * self.r;
    if det < 0.0 {
      None
    } else {
      let det = det.sqrt();
      if b - det > EPS { Some(b - det) } else {
        if b + det > EPS { Some(b + det) } else {
          None // solved to t<0, so the sphere is behind the ray, consider it as no hit
        }
      }.map(|t| {
        let p = ray.o + ray.d * t - self.c;
        let norm = p.norm();
//        u = 0.5 + arctan2(dz, dx) / (2*pi)
//        v = 0.5 - arcsin(dy) / pi
        let uv = Vec2(0.5 + norm.2.atan2(norm.0) / (2.0 * PI), 0.5 - norm.1.asin() / PI);
//        let uv = Vec2((norm.1 / norm.0).atan() / (PI / 2.0), norm.2.acos() / PI);
//        let uv = Vec2(uv.0.mod_euc(1.0), uv.1.mod_euc(1.0));
        HitResult { t, norm, uv }
      })
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct InfPlane {
  // any point on the plane
  pub p: Vec3,
  // norm vec
  pub n: Vec3,
}

impl InfPlane {
  pub fn new(p: Vec3, n: Vec3) -> InfPlane {
    InfPlane { p, n: n.norm() }
  }

  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    // let: u,v be a pair of orthogonal basis of the plane
    // solve: p + a*u + b*v = o + t*d
    // dot n on both side => dot(p-o,n)=t*dot(d,n)
    let dot_d_n = ray.d.dot(self.n);
    if dot_d_n.abs() < EPS {
      None
    } else {
      let t = (self.p - ray.o).dot(self.n) / dot_d_n;
      if t < EPS { None } else {
        Some(HitResult { t, norm: self.n, uv: Vec2::zero() })
      }
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Circle {
  // plane.p is the center of circle
  pub plane: InfPlane,
  // u, v are to help to use r, theta to determine a point on circle
  pub u: Vec3,
  pub v: Vec3,
}

impl Circle {
  pub fn new(c: Vec3, n: Vec3, r: f32) -> Circle {
    let plane = InfPlane::new(c, n);
    let u = plane.n.get_orthogonal_for_unit() * r;
    let v = plane.n.cross(u).norm() * r;
    Circle { plane, u, v }
  }

  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    if let Some(t) = self.plane.hit(ray) {
      if (ray.o + ray.d * t.t - self.plane.p).len2() < self.u.len2() {
        Some(t)
      } else { None }
    } else { None }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Rectangle {
  // plane.p is the cross point of u and v
  pub plane: InfPlane,
  pub u: Vec3,
  pub inv_u_len: f32,
  pub v: Vec3,
  pub inv_v_len: f32,
}

impl Rectangle {
  pub fn new(p: Vec3, u: Vec3, v: Vec3) -> Rectangle {
    assert!(u.dot(v).abs() < EPS);
    Rectangle { plane: InfPlane::new(p, u.cross(v)), u: u.norm(), inv_u_len: 1.0 / u.len(), v: v.norm(), inv_v_len: 1.0 / v.len() }
  }

  fn hit(&self, ray: &Ray) -> Option<HitResult> {
    if let Some(t) = self.plane.hit(ray) {
      let p = ray.o + ray.d * t.t - self.plane.p;
      let u = p.dot(self.u) * self.inv_u_len;
      let v = p.dot(self.v) * self.inv_v_len;
      if EPS < u && u < 1.0 - EPS && EPS < v && v < 1.0 - EPS {
        Some(HitResult { uv: Vec2(u, v), ..t })
      } else { None }
    } else { None }
  }
}

#[derive(Serialize, Deserialize)]
pub enum Geo {
  Sphere(Sphere),
  InfPlane(InfPlane),
  Circle(Circle),
  Rectangle(Rectangle),
  Mesh(Mesh),
  RotateBezier(RotateBezier),
}

impl Geo {
  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    match self {
      Geo::Sphere(sphere) => sphere.hit(ray),
      Geo::InfPlane(inf_plane) => inf_plane.hit(ray),
      Geo::Circle(circle) => circle.hit(ray),
      Geo::Rectangle(rectangle) => rectangle.hit(ray),
      Geo::Mesh(mesh) => mesh.hit(ray),
      Geo::RotateBezier(bezier) => bezier.hit(ray),
    }
  }
}


#[derive(Serialize, Deserialize)]
pub struct Object {
  pub geo: Geo,
  pub color: Color,
  pub texture: Texture,
}

#[derive(Serialize, Deserialize)]
pub enum LightGeo {
  Circle(Circle),
}

impl LightGeo {
  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    match self {
      LightGeo::Circle(circle) => circle.hit(ray),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct LightSource {
  pub geo: LightGeo,
  pub emission: Vec3,
}