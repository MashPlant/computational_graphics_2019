use super::util::*;
use super::mesh::*;
use super::geo::*;
use super::vec::*;
use serde::{Serialize, Deserialize};

// ub: upper bound
// lower bound is the index of this node
// [lower bound, upper bound) is the triangles that lie across the split line
// index -1 => null
#[derive(Copy, Clone, Serialize, Deserialize)]
struct KDNode {
  aabb: AABB,
  // the aabb of triangles cross the split line
  cross_aabb: AABB,
  // will split at the d with biggest variance, instead of regularly 012012...
  split: f32,
  d: u32,
}

#[derive(Serialize, Deserialize)]
pub struct KDTree {
  nodes: Box<[KDNode]>,
  index: Box<[(u32, u32, u32)]>,
}

const LEAF_SIZE: usize = 8;

impl KDTree {
  pub fn new(index: &[(u32, u32, u32)], v: &[Vec3]) -> KDTree {
    let len  = index.len();
    let mut not_built_yet = KDTree {
      nodes: vec![KDNode { aabb: AABB { min: Vec3::zero(), max: Vec3::zero() }, cross_aabb: AABB { min: Vec3::zero(), max: Vec3::zero() }, split: 0.0, d: 0 };
                  len].into(),
      index: index.into(),
    };
    not_built_yet.build(v, 0, len, 0);
    not_built_yet
  }

  fn build(&mut self, v: &[Vec3], l: usize, r: usize, dep: u32) {
    macro_rules! min {
      ($i: expr, $j: expr, $k: expr, $d: expr) => { v[$i as usize][$d].min(v[$j as usize][$d]).min(v[$k as usize][$d]) };
    }
    macro_rules! at {
      ($i: expr) => { v[$i as usize] };
    }
    let update = |min: &mut Vec3, max: &mut Vec3, i: u32, j: u32, k: u32| {
      min.0 = min.0.min(at!(i).0).min(at!(j).0).min(at!(k).0);
      min.1 = min.1.min(at!(i).1).min(at!(j).1).min(at!(k).1);
      min.2 = min.2.min(at!(i).2).min(at!(j).2).min(at!(k).2);
      max.0 = max.0.max(at!(i).0).max(at!(j).0).max(at!(k).0);
      max.1 = max.1.max(at!(i).1).max(at!(j).1).max(at!(k).1);
      max.2 = max.2.max(at!(i).2).max(at!(j).2).max(at!(k).2);
    };
    let mid = (l + r) / 2;
    if r - l <= LEAF_SIZE {
      self.nodes[mid].aabb = {
        let mut min = Vec3(1e9, 1e9, 1e9);
        let mut max = Vec3(-1e9, -1e9, -1e9);
        for &(i, j, k) in &self.index[l..r] {
          update(&mut min, &mut max, i, j, k);
        }
        AABB { min, max }
      };
      return;
    }
    let d = {
      let (mut split_d, mut max_var) = (0, -1e9);
      for d in 0..3 {
        // use f64 for precision
        let (mut ave, mut var) = (0.0, 0.0);
        for &(i, j, k) in &self.index[l..r] {
          ave += min!(i, j, k, d) as f64;
        }
        ave /= (r - l) as f64;
        for &(i, j, k) in &self.index[l..r] {
          var += (min!(i, j, k, d) as f64 as f64 - ave).powi(2);
        }
        if var > max_var {
          split_d = d;
          max_var = var;
        }
      }
      split_d
    };
    self.index[l..r].sort_by(|&(i1, j1, k1), &(i2, j2, k2)| {
      min!(i1, j1, k1, d).partial_cmp(&min!(i2, j2, k2, d)).unwrap()
    });
    let split = min!(self.index[mid].0, self.index[mid].1, self.index[mid].2, d);
    self.nodes[mid] = KDNode {
      aabb: {
        let mut min = Vec3(1e9, 1e9, 1e9);
        let mut max = Vec3(-1e9, -1e9, -1e9);
        for &(i, j, k) in &self.index[l..r] {
          update(&mut min, &mut max, i, j, k);
        }
        AABB { min, max }
      },
      cross_aabb: {
        let mut min = Vec3(1e9, 1e9, 1e9);
        let mut max = Vec3(-1e9, -1e9, -1e9);
        for &(i, j, k) in &self.index[l..mid] { // cross triangles are only in left subtree
          if v[i as usize][d].max(v[j as usize][d]).max(v[k as usize][d]) > split {
            update(&mut min, &mut max, i, j, k);
          }
        }
        if min.0 > max.0 { max = min; }
        AABB { min, max }
      },
      split,
      d: d as u32,
    };
    self.build(v, l, mid, dep + 1);
    self.build(v, mid, r, dep + 1);
  }

  pub fn hit(&self, ray: &Ray, mesh: &Mesh) -> Option<HitResult> {
    self.hit_impl(ray, Vec3(1.0 / ray.d.0, 1.0 / ray.d.1, 1.0 / ray.d.2), mesh, 0, self.index.len())
  }

  fn hit_impl(&self, ray: &Ray, inv_d: Vec3, mesh: &Mesh, l: usize, r: usize) -> Option<HitResult> {
    let mid = (l + r) / 2;
    let KDNode { aabb, cross_aabb, d, split } = self.nodes[mid];
    if !aabb.intersect_ray(ray.o, inv_d) { return None; }
    if r - l <= LEAF_SIZE {
      let mut cur_t = 1e9;
      let mut ret = None;
      for &(i, j, k) in &self.index[l..r] {
        let (i, j, k) = (i as usize, j as usize, k as usize);
        let (p1, p2, p3) = (mesh.v[i], mesh.v[j], mesh.v[k]);
        let (e1, e2) = (p2 - p1, p3 - p1);
        let p = ray.d.cross(e2);
        let det = e1.dot(p);
        if det.abs() < EPS { continue; } // parallel to plane
        let inv_det = 1.0 / det;
        let t = ray.o - p1;
        let u = t.dot(p) * inv_det;
        if u < 0.0 || u > 1.0 { continue; } // intersect outside the triangle
        let q = t.cross(e1);
        let v = ray.d.dot(q) * inv_det;
        if v < 0.0 || u + v > 1.0 { continue; } // intersect outside the triangle
        let t = e2.dot(q) * inv_det;
        if t > EPS && t < cur_t {
          cur_t = t;
          let (n1, n2, n3) = (mesh.norm[i], mesh.norm[j], mesh.norm[k]);
          let norm = n1 * (1.0 - u - v) + n2 * u + n3 * v;
          let (uv1, uv2, uv3) = (mesh.uv[i], mesh.uv[j], mesh.uv[k]);
          let uv = uv1 * (1.0 - u - v) + uv2 * u + uv3 * v;
          ret = Some(HitResult { t, norm, uv });
        }
      }
      ret
    } else {
      let d = d as usize;
      if let Some(t) = cross_aabb.hit(ray.o, inv_d) {
        let p = ray.o + ray.d * t;
        if p[d] < cross_aabb.min[d] {
          self.hit_impl(ray, inv_d, mesh, l, mid).or_else(||
            self.hit_impl(ray, inv_d, mesh, mid, r))
        } else if p[d] > cross_aabb.max[d] {
          self.hit_impl(ray, inv_d, mesh, mid, r).or_else(||
            self.hit_impl(ray, inv_d, mesh, l, mid))
        } else {
          let r1 = self.hit_impl(ray, inv_d, mesh, l, mid);
          let r2 = self.hit_impl(ray, inv_d, mesh, mid, r);
          match (r1, r2) {
            (None, r2) => r2,
            (r1, None) => r1,
            (Some(r1), Some(r2)) => if r1.t < r2.t { Some(r1) } else { Some(r2) }
          }
        }
      } else {
        if ray.o[d] < split {
          self.hit_impl(ray, inv_d, mesh, l, mid).or_else(||
            self.hit_impl(ray, inv_d, mesh, mid, r))
        } else {
          self.hit_impl(ray, inv_d, mesh, mid, r).or_else(||
            self.hit_impl(ray, inv_d, mesh, l, mid))
        }
      }
    }
  }
}