use super::util::*;
use super::mesh::*;
use super::geo::*;
use super::vec::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum NodeKind {
  // instead of [Box<OctNode>; 8], I think Box<[OctNode; 8]> is better
  Internal(Box<[OctNode; 8]>),
  // internal node doesn't need to hold index, only need to check index of leaf node
  Leaf(Box<[(u32, u32, u32)]>),
}

#[derive(Serialize, Deserialize)]
pub struct OctNode {
  pub aabb: AABB,
  pub kind: NodeKind,
}

// prob seq for oct tree hit detection
// the index the group before has higher priority, if it says `hit`, no need to check idx after
// this is actually an order of [number of different bits]
pub const PROB: [[u8; 8]; 8] = [
  [0, 1, 2, 4, 3, 5, 6, 7],
  [1, 0, 3, 5, 2, 4, 7, 6],
  [2, 0, 3, 6, 1, 4, 7, 5],
  [3, 1, 2, 7, 0, 5, 6, 4],
  [4, 0, 5, 6, 1, 2, 7, 3],
  [5, 1, 4, 7, 0, 3, 6, 2],
  [6, 2, 4, 7, 0, 3, 5, 1],
  [7, 3, 5, 6, 1, 2, 4, 0],
];

impl OctNode {
  pub fn new(index: &[(u32, u32, u32)], v: &[Vec3], min: Vec3, max: Vec3, dep: u32) -> OctNode {
    let aabb = AABB { min, max };
    let mut self_index = Vec::with_capacity(v.len());
    for (i, j, k) in index {
      if aabb.intersect_triangle(v[*i as usize], v[*j as usize], v[*k as usize]) {
        self_index.push((*i, *j, *k))
      }
    }
    let kind = if dep == 0 || self_index.len() < 16 {
      NodeKind::Leaf(self_index.into())
    } else {
      let dep = dep - 1;
      let mid = (min + max) / 2.0;
      // ch_index = sigma (ch_min[i] == mid[i]) * 2^i
      NodeKind::Internal(Box::new([
        OctNode::new(&self_index, v, min, mid, dep),
        OctNode::new(&self_index, v, Vec3(mid.0, min.1, min.2), Vec3(max.0, mid.1, mid.2), dep),
        OctNode::new(&self_index, v, Vec3(min.0, mid.1, min.2), Vec3(mid.0, max.1, mid.2), dep),
        OctNode::new(&self_index, v, Vec3(mid.0, mid.1, min.2), Vec3(max.0, max.1, mid.2), dep),
        OctNode::new(&self_index, v, Vec3(min.0, min.1, mid.2), Vec3(mid.0, mid.1, max.2), dep),
        OctNode::new(&self_index, v, Vec3(mid.0, min.1, mid.2), Vec3(max.0, mid.1, max.2), dep),
        OctNode::new(&self_index, v, Vec3(min.0, mid.1, mid.2), Vec3(mid.0, max.1, max.2), dep),
        OctNode::new(&self_index, v, mid, max, dep),
      ]))
    };
    OctNode { aabb, kind }
  }

  pub fn hit(&self, ray: &Ray, mesh: &Mesh) -> Option<HitResult> {
    self.hit_impl(ray, Vec3(1.0 / ray.d.0, 1.0 / ray.d.1, 1.0 / ray.d.2), mesh)
  }

  pub fn hit_impl(&self, ray: &Ray, inv_d: Vec3, mesh: &Mesh) -> Option<HitResult> {
    match &self.kind {
      NodeKind::Internal(children) => {
        if !self.aabb.intersect_ray(ray.o, inv_d) { return None; }
        let children = children.as_ref();
        let mid = (self.aabb.min + self.aabb.max) / 2.0;
        let index = ((ray.o.0 > mid.0) as usize) + (((ray.o.1 > mid.1) as usize) << 1) + (((ray.o.2 > mid.2) as usize) << 2);
        for child_idx in &PROB[index] {
          if let Some(result) = children[*child_idx as usize].hit_impl(ray, inv_d, mesh) {
            return Some(result);
          }
        }
        None
      }
      NodeKind::Leaf(idx) => {
        if idx.len() == 0 || !self.aabb.intersect_ray(ray.o, inv_d) { return None; }
        let mut cur_t = 1e9;
        let mut ret = None;
        for (i, j, k) in idx.as_ref() {
          let (i, j, k) = (*i as usize, *j as usize, *k as usize);
          let (p1, p2, p3) = (mesh.v[i], mesh.v[j], mesh.v[k]);
          let (e1, e2) = (p2 - p1, p3 - p1);
          let p = ray.d.cross(e2);
          let det = e1.dot(p);
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
      }
    }
  }

  pub fn hit_no_rec(&self, ray: &Ray, mesh: &Mesh) -> Option<HitResult> {
    use std::mem;
    unsafe {
      let inv_d = Vec3(1.0 / ray.d.0, 1.0 / ray.d.1, 1.0 / ray.d.2);
      let mut stk: [&OctNode; 56] = mem::uninitialized();
      let mut top = 0;
      macro_rules! push {
        ($x: expr) => {
          *stk.get_unchecked_mut(top) = $x;
          top += 1;
        };
      }
      macro_rules! pop {
        () => {{
          top -= 1;
          *stk.get_unchecked(top)
        }};
      }
      push!(self);
      while top != 0 {
        let mut x = pop!();
        let ret = loop {
          match &x.kind {
            NodeKind::Internal(ch) => {
              if !x.aabb.intersect_ray(ray.o, inv_d) {
                break None;
              }
              let mid = (x.aabb.min + x.aabb.max) / 2.0;
              let index = ((ray.o.0 > mid.0) as usize) + (((ray.o.1 > mid.1) as usize) << 1) + (((ray.o.2 > mid.2) as usize) << 2);
              let prob = PROB.get_unchecked(index);
              push!(&ch[prob[7] as usize]);
              push!(&ch[prob[6] as usize]);
              push!(&ch[prob[5] as usize]);
              push!(&ch[prob[4] as usize]);
              push!(&ch[prob[3] as usize]);
              push!(&ch[prob[2] as usize]);
              push!(&ch[prob[1] as usize]);
              x = &ch[prob[0] as usize];
            }
            NodeKind::Leaf(idx) => {
              if idx.len() == 0 || !x.aabb.intersect_ray(ray.o, inv_d) {
                break None;
              }
              let mut cur_t = 1e9;
              let mut ret = None;
              for (i, j, k) in idx.as_ref() {
                let (i, j, k) = (*i as usize, *j as usize, *k as usize);
                let (&p1, &p2, &p3) = (mesh.v.get_unchecked(i), mesh.v.get_unchecked(j), mesh.v.get_unchecked(k));
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
                  let (&n1, &n2, &n3) = (mesh.norm.get_unchecked(i), mesh.norm.get_unchecked(j), mesh.norm.get_unchecked(k));
                  let norm = n1 * (1.0 - u - v) + n2 * u + n3 * v;
                  let (&uv1, &uv2, &uv3) = (mesh.uv.get_unchecked(i), mesh.uv.get_unchecked(j), mesh.uv.get_unchecked(k));
                  let uv = uv1 * (1.0 - u - v) + uv2 * u + uv3 * v;
                  ret = Some(HitResult { t, norm, uv });
                }
              }
              break ret;
            }
          }
        };
        if !ret.is_none() {
          return ret;
        }
      }
      None
    }
  }
}
