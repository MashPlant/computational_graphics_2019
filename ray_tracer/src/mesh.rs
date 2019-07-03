use super::vec::*;
use super::geo::*;
use super::tri_aabb::*;
use super::mat44::*;
use super::kd_tree::KDNode;
use serde::{Serialize, Deserialize};

// compared to Vec<T>, Box<[T]> is smaller(but not as small as a raw pointer, Box<[T]> has a size of sizeof(ptr) * 2)
// and we don't need dynamic allocation during running

// if I was using rust for rendering, I would definitely design the data structure more carefully
// but now rust is ony for code gen, so I don't need to care too much about efficiency
#[derive(Serialize, Deserialize)]
pub struct Mesh {
  pub v: Box<[Vec3]>,
  // uv is the parameter in parameter function
  pub uv: Box<[Vec2]>,
  pub norm: Box<[Vec3]>,
  pub kd: KDNode,
//  pub oct: OctNode,
//  pub tree: Box<OctNode>,
}

impl Mesh {
  pub fn new(v: Vec<Vec3>, uv: Vec<Vec2>, norm: Vec<Vec3>, mut index: Vec<(u32, u32, u32)>) -> Mesh {
//    let AABB { min, max } = AABB::from_slice(&v);
//    let oct = OctNode::new(&index, &v, min, max, 8);
    let kd = KDNode::new(&mut index, &v, 16);
    Mesh {
      v: v.into(),
      uv: uv.into(),
      norm: norm.into(),
      kd,
//      oct,
    }
  }

  pub fn hit(&self, ray: &Ray) -> Option<HitResult> {
    self.kd.short_stack(ray, self)
  }
}

// curve on xy plain
pub trait Curve {
  // return (Vec2(x, y), Vec2(dx, dy)) at parameter t
  // 0 <= t <= 1
  fn eval(&self, t: f32) -> (Vec2, Vec2);

  // rotate the curve around y-axis to get a surface, then apply transform to the surface
  fn to_mesh(&self, sample_t: usize, sample_theta: usize, transform: Mat44) -> Mesh {
    (|theta: f32, t: f32| -> (Vec3, Vec3) {
      use std::f32::consts::PI;
      let theta = theta * 2.0 * PI;
      let (cos, sin) = (theta.cos(), theta.sin());
      let (value, tangent) = self.eval(t);
      let point = (transform * Vec4(value.0 * cos, value.1, -value.0 * sin, 1.0)).to_vec3();
      let tangent_t = (transform * Vec4(tangent.0 * cos, tangent.1, -tangent.0 * sin, 0.0)).to_vec3();
      let tangent_theta = (transform * Vec4(-value.0 * sin, 0.0, -value.0 * cos, 0.0)).to_vec3();
      let norm = tangent_t.cross(tangent_theta).norm();
//      let norm = if tangent_theta.len2() < EPS * EPS { // this happens when original x = 0
//        let xz = Vec2(tangent_t.0, tangent_t.2);
//        let xz_len = xz.len();
//        if tangent_t.1 < 0.0 {
//          let xz = xz / xz_len * -tangent_t.1;
//          Vec3(xz.0, xz_len, xz.1)
//        } else {
//          let xz = xz / xz_len * tangent_t.1;
//          Vec3(xz.0, -xz_len, xz.1)
//        }
//      } else { tangent_theta.cross(tangent_t) };
// guarantee norm vector point to the out of surface
//      let norm = if (point + norm * EPS).len2() < point.len2() {
//        println!("neg");
//        -norm
//      } else {
//        println!("pos");
//        norm
//      };
      (point, norm)
    }).to_mesh(sample_t, sample_theta, Mat44::identity()) // transform is already applied above
  }
}

impl<T: Fn(f32) -> (Vec2, Vec2)> Curve for T {
  fn eval(&self, t: f32) -> (Vec2, Vec2) {
    self(t)
  }
}

pub trait Surface {
  // return (Vec3(x, y, z), norm) at parameter (u, v)
  // 0 <= u, v <= 1
  fn eval(&self, u: f32, v: f32) -> (Vec3, Vec3);

  fn to_mesh(&self, sample_u: usize, sample_v: usize, transform: Mat44) -> Mesh {
    assert!(sample_u > 1);
    assert!(sample_v > 1);
    let (d_u, d_v) = (1.0 / (sample_u - 1) as f32, 1.0 / (sample_v - 1) as f32);
    let (mut vs, mut uvs, mut norms, mut indices) = (Vec::with_capacity(sample_u * sample_v), Vec::with_capacity(sample_u * sample_v), Vec::with_capacity(sample_u * sample_v), Vec::with_capacity(sample_u * sample_v * 2));
    let mut u = 0.0;
    for i in 0..sample_v {
      let mut v = 0.0;
      for j in 0..sample_u {
        let (point, norm) = self.eval(u, v);
        vs.push((transform * point.extend(1.0)).to_vec3());
        uvs.push(Vec2(u, v));
        norms.push((transform * norm.extend(0.0)).to_vec3().norm());
        if i > 0 && j > 0 {
          let total = (i * sample_u + j) as u32;
          // make two triangle from a rectangle
//          if vs[total as usize - 1 - sample_u] != vs[total as usize - 1] {
          indices.push((total, total - 1 - sample_u as u32, total - 1));
//          }
          indices.push((total, total - sample_u as u32, total - 1 - sample_u as u32));
        }
        v += d_v;
      }
      u += d_u;
    }
    // wrap back
//    for j in 1..sample_u {
//      let last = ((sample_v - 1) * sample_u + j) as u32;
////      if vs[last as usize - 1] != vs[j - 1] {
//      indices.push((j as u32, last - 1, j as u32 - 1));
////      }
//      indices.push((j as u32, last, last - 1));
//    }
    Mesh::new(vs, uvs, norms, indices)
  }
}

impl<T: Fn(f32, f32) -> (Vec3, Vec3)> Surface for T {
  fn eval(&self, u: f32, v: f32) -> (Vec3, Vec3) {
    self(u, v)
  }
}

// axis-aligned minimum bounding box
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct AABB {
  pub min: Vec3,
  pub max: Vec3,
}

impl AABB {
  // like in InfPlane, to get the intersection point of ray and plane,
  // use (plane.p - ray.o).dot(plane.n) / ray.d.dot(plane.n)
  // here n is (1, 0, 0) or something like it, so (p - o).schur(inv_d) contain 3 intersection points
  // a pair of (t0s.x, t1s.x) is the intersection points on two facing surface
  // recompute inv d to be faster
  pub fn intersect_ray(&self, o: Vec3, inv_d: Vec3) -> bool {
    !self.hit(o, inv_d).is_none()
  }

  pub fn hit(self, o: Vec3, inv_d: Vec3) -> Option<(f32, f32)> {
    let t0s = (self.min - o).schur(inv_d);
    let t1s = (self.max - o).schur(inv_d);
    let t_min = (t0s.0.min(t1s.0)).max(t0s.1.min(t1s.1)).max(t0s.2.min(t1s.2));
    let t_max = (t0s.0.max(t1s.0)).min(t0s.1.max(t1s.1)).min(t0s.2.max(t1s.2));
    if t_min.max(0.0) < t_max { Some((t_min, t_max)) } else { None }
  }

  // Separating Axis Theorem
  // for any two convex objects, if there exists an axis that separates them, no collision occurs
  pub fn intersect_triangle(&self, p1: Vec3, p2: Vec3, p3: Vec3) -> bool {
    let mid = (self.min + self.max) / 2.0;
    let half = self.max - mid;
    tri_box_overlap([mid.0, mid.1, mid.2], [half.0, half.1, half.2],
                    [[p1.0, p1.1, p1.2], [p2.0, p2.1, p2.2], [p3.0, p3.1, p3.2]])
  }

  pub fn from_slice(s: &[Vec3]) -> AABB {
    assert!(s.len() >= 1);
    let mut min = Vec3(1e9, 1e9, 1e9);
    let mut max = Vec3(-1e9, -1e9, -1e9);
    for v in s {
      min.0 = min.0.min(v.0);
      min.1 = min.1.min(v.1);
      min.2 = min.2.min(v.2);
      max.0 = max.0.max(v.0);
      max.1 = max.1.max(v.1);
      max.2 = max.2.max(v.2);
    }
    AABB { min, max }
  }
}