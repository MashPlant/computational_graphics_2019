use super::util::*;
use super::mesh::*;
use super::geo::*;
use super::vec::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KDNode {
  pub aabb: AABB,
  pub kind: KDNodeKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KDNodeKind {
  Internal(Box<[KDNode; 2]>, u32, f32),
  Leaf(Box<[(u32, u32, u32)]>),
}

impl KDNode {
  pub fn new(index: &mut [(u32, u32, u32)], v: &[Vec3], dep: u32) -> KDNode {
    let aabb = {
      let mut min = Vec3(1e9, 1e9, 1e9);
      let mut max = Vec3(-1e9, -1e9, -1e9);
      for &(i, j, k) in index.iter() {
        min.0 = min.0.min(v[i as usize].0).min(v[j as usize].0).min(v[k as usize].0);
        min.1 = min.1.min(v[i as usize].1).min(v[j as usize].1).min(v[k as usize].1);
        min.2 = min.2.min(v[i as usize].2).min(v[j as usize].2).min(v[k as usize].2);
        max.0 = max.0.max(v[i as usize].0).max(v[j as usize].0).max(v[k as usize].0);
        max.1 = max.1.max(v[i as usize].1).max(v[j as usize].1).max(v[k as usize].1);
        max.2 = max.2.max(v[i as usize].2).max(v[j as usize].2).max(v[k as usize].2);
      }
      AABB { min, max }
    };
    if dep == 0 || index.len() < 16 {
      KDNode { aabb, kind: KDNodeKind::Leaf(Box::from(&*index)) }
    } else {
      macro_rules! min {
        ($i: expr, $j: expr, $k: expr, $d: expr) => { v[$i as usize][$d].min(v[$j as usize][$d]).min(v[$k as usize][$d]) };
      }
      macro_rules! max {
        ($i: expr, $j: expr, $k: expr, $d: expr) => { v[$i as usize][$d].max(v[$j as usize][$d]).max(v[$k as usize][$d]) };
      }
      let sp_d = {
        let (mut sp_d, mut max_var) = (0, -1e9);
        for d in 0..3 {
          let (mut ave, mut var) = (0.0, 0.0);
          for &(i, j, k) in index.iter() { ave += min!(i, j, k, d) as f64; }
          ave /= index.len() as f64;
          for &(i, j, k) in index.iter() { var += (min!(i, j, k, d) as f64 as f64 - ave).powi(2); }
          if var > max_var { (sp_d = d, max_var = var); }
        }
        sp_d
      };
      index.sort_by(|&(i1, j1, k1), &(i2, j2, k2)| {
        min!(i1, j1, k1, sp_d).partial_cmp(&min!(i2, j2, k2, sp_d)).unwrap()
      });
      let (mid_i, mid_j, mid_k) = index[index.len() / 2];
      let sp = min!(mid_i, mid_j, mid_k, sp_d);
      let (mut l, mut r) = (Vec::new(), Vec::new());
      for &(i, j, k) in index.iter() {
        if min!(i, j, k, sp_d) < sp { l.push((i, j, k)); }
        if max!(i, j, k, sp_d) > sp { r.push((i, j, k)); }
      }
      if l.len() == index.len() || r.len() == index.len() {
        KDNode { aabb, kind: KDNodeKind::Leaf(Box::from(&*index)) }
      } else {
        KDNode { aabb, kind: KDNodeKind::Internal(Box::new([KDNode::new(&mut l, v, dep - 1), KDNode::new(&mut r, v, dep - 1)]), sp_d as u32, sp) }
      }
    }
  }

  pub fn short_stack(&self, ray: &Ray, mesh: &Mesh) -> Option<HitResult> {
    let mut this = self;
    let inv_d = Vec3(1.0 / ray.d.0, 1.0 / ray.d.1, 1.0 / ray.d.2);
    if let Some((root_min, root_max)) = this.aabb.hit(ray.o, inv_d) {
      let mut t_min;
      let (mut t_max, mut ret_t, mut ret) = (root_min, 1e9, None);
      let mut stk = vec![];
      while t_max < root_max {
        let (mut node, mut push_down) = match stk.pop() {
          Some((pop_x, pop_t_min, pop_t_max)) => {
            t_min = pop_t_min;
            t_max = pop_t_max;
            (pop_x, false)
          }
          None => {
            t_min = t_max;
            t_max = root_max;
            (this, true)
          }
        };
        while node.aabb.intersect_ray(ray.o, inv_d) {
          match &node.kind {
            KDNodeKind::Internal(ch, sp_d, sp) => {
              let (sp_d, sp) = (*sp_d as usize, *sp);
              let t_sp = (sp - ray.o[sp_d]) / ray.d[sp_d];
              let (fst, snd) = if ray.d[sp_d] > 0.0 { (&ch[0], &ch[1]) } else { (&ch[1], &ch[0]) };
              if t_sp <= t_min {
                node = snd;
              } else if t_sp >= t_max {
                node = fst;
              } else {
                stk.push((snd, t_sp, t_max));
                node = fst;
                t_max = t_sp;
                push_down = false;
              }
              if push_down {
                this = node;
              }
            }
            KDNodeKind::Leaf(indices) => {
              for (i, j, k) in indices.as_ref() {
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
                if t > EPS && t < ret_t {
                  ret_t = t;
                  let (n1, n2, n3) = (mesh.norm[i], mesh.norm[j], mesh.norm[k]);
                  let norm = n1 * (1.0 - u - v) + n2 * u + n3 * v;
                  let (uv1, uv2, uv3) = (mesh.uv[i], mesh.uv[j], mesh.uv[k]);
                  let uv = uv1 * (1.0 - u - v) + uv2 * u + uv3 * v;
                  ret = Some(HitResult { t, norm, uv });
                  if t < t_max {
                    return ret;
                  }
                }
              }
              break;
            }
          }
        }
      }
      ret
    } else { None }
  }

  pub fn hit_no_rec(&self, ray: &Ray, mesh: &Mesh) -> Option<HitResult> {
    let inv_d = Vec3(1.0 / ray.d.0, 1.0 / ray.d.1, 1.0 / ray.d.2);
    if let Some((root_min, root_max)) = self.aabb.hit(ray.o, inv_d) {
      let mut stk = vec![(self, root_min, root_max)];
      let (mut ret_t, mut ret) = (1e9, None);
      while let Some((mut node, t_min, mut t_max)) = stk.pop() {
        while node.aabb.intersect_ray(ray.o, inv_d) {
          match &node.kind {
            KDNodeKind::Internal(ch, sp_d, sp) => {
              let (sp_d, sp) = (*sp_d as usize, *sp);
              let t_sp = (sp - ray.o[sp_d]) / ray.d[sp_d];
              let (fst, snd) = if ray.d[sp_d] > 0.0 { (&ch[0], &ch[1]) } else { (&ch[1], &ch[0]) };
              if t_sp <= t_min {
                node = snd;
              } else if t_sp >= t_max {
                node = fst;
              } else {
                stk.push((snd, t_sp, t_max));
                node = fst;
                t_max = t_sp;
              }
            }
            KDNodeKind::Leaf(indices) => {
              for (i, j, k) in indices.as_ref() {
                let (i, j, k) = (*i as usize, *j as usize, *k as usize);
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
                if t > EPS && t < ret_t {
                  ret_t = t;
                  let (n1, n2, n3) = (mesh.norm[i], mesh.norm[j], mesh.norm[k]);
                  let norm = n1 * (1.0 - u - v) + n2 * u + n3 * v;
                  let (uv1, uv2, uv3) = (mesh.uv[i], mesh.uv[j], mesh.uv[k]);
                  let uv = uv1 * (1.0 - u - v) + uv2 * u + uv3 * v;
                  ret = Some(HitResult { t, norm, uv });
                  if t < t_max {
                    return ret;
                  }
                }
              }
              break;
            }
          }
        }
      }
      ret
    } else { None }
  }
}