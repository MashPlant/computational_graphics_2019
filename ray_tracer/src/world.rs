use super::geo::*;
use super::pic::*;
use super::vec::*;
use super::material::*;
use super::util::*;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct World {
  pub objs: Vec<Object>,
  pub light: LightSource,
  pub env: Vec3,
  pub cam: Ray,
  pub w: u32,
  pub h: u32,
}

impl World {
  pub fn from_json(json: &str) -> serde_json::Result<World> {
    serde_json::from_str(json)
  }

  pub fn to_json(&self) -> serde_json::Result<String> {
    serde_json::to_string(self)
  }

  pub fn from_bincode(bincode: &[u8]) -> bincode::Result<World> {
    bincode::deserialize(bincode)
  }

  pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
    bincode::serialize(self)
  }
}


// util functions
impl World {
  fn prologue(&self) -> (Vec<Vec3>, Vec3, Vec3, Instant, Arc<Mutex<(u32, Instant)>>) {
    let buf = vec![Vec3::zero(); (self.w * self.h) as usize];
    let cx = Vec3(self.w as f32 * 0.5135 / self.h as f32, 0.0, 0.0);
    let cy = cx.cross(self.cam.d).norm() * 0.5135;
    let start = Instant::now();
    (buf, cx, cy, start, Arc::new(Mutex::new((0, start))))
  }

  fn print_progress(progress: u32, start: Instant, now: &mut Instant) {
    let elapsed = start.elapsed().as_secs_f32();
    let last_iter = now.elapsed().as_secs_f32();
    *now = Instant::now();
    eprintln!("rendering {:>02}%, elapsed {:.3}s, last iter {:.3}s, remain {:.3}s", progress, elapsed, last_iter,
              last_iter * (100.0 - progress as f32));
  }
}

// path tracing
impl World {
  pub fn path_tracing(&self, sample: u32) -> PNG {
    let (mut buf, cx, cy, start, progress) = self.prologue();
    buf.par_iter_mut().enumerate().for_each(|(index, color)| {
      let index = index as u32;
      // index = (h-y-1)*w+x
      let (x, y) = ((index % self.w) as f32, (self.h - 1 - index / self.w) as f32);
      let mut rng = XorShiftRng::new(index);
      let mut color_t = Vec3::zero();
      for _ in 0..sample / 4 { // because we will do 4-super sample later
        for sx in 0..2 {
          for sy in 0..2 {
            let (r1, r2) = (2.0 * rng.gen(), 2.0 * rng.gen());
            let dx = if r1 < 1.0 { r1.sqrt() - 1.0 } else { 1.0 - (2.0 - r1).sqrt() };
            let dy = if r2 < 1.0 { r2.sqrt() - 1.0 } else { 1.0 - (2.0 - r2).sqrt() };
            let d = cx * (((sx as f32 + 0.5 + dx) * 0.5 + x) / self.w as f32 - 0.5)
              + cy * (((sy as f32 + 0.5 + dy) * 0.5 + y) / self.h as f32 - 0.5) + self.cam.d;
            color_t += self.path_tracing_impl(Ray::new(self.cam.o + d * 14.0, d), 0, &mut rng) / sample as f32;
          }
        }
      }
      *color = Vec3(clamp(color_t.0), clamp(color_t.1), clamp(color_t.2));
      let mut progress = progress.lock().unwrap();
      progress.0 += 1;
      if progress.0 % (self.w * self.h / 100) == 0 {
        World::print_progress(progress.0 / (self.w * self.h / 100), start, &mut progress.1);
      }
    });
    PNG::new(&buf, self.w, self.h)
  }

  fn hit_all(&self, ray: &Ray) -> Option<(HitResult, Option<&Object>, Vec3)> {
    let mut t = 1e9;
    let mut info = None;
    for obj in &self.objs {
      if let Some(result) = obj.geo.hit(ray) {
        if result.t < t {
          t = result.t;
          info = Some((result, Some(obj)));
        }
      }
    }
    let emission = if let Some(result) = self.light.geo.hit(ray) {
      if result.t < t {
        info = Some((result, None));
        self.light.emission
      } else { Vec3::zero() }
    } else { Vec3::zero() };
    info.map(|(result, obj)| {
      (result, obj, emission)
    })
  }

  fn path_tracing_impl(&self, ray: Ray, dep: u32, rng: &mut XorShiftRng) -> Vec3 {
    if dep >= 5 { return self.env; }
    if let Some(hit_result) = self.hit_all(&ray) {
      let (HitResult { t, norm, uv }, obj, emission) = hit_result;
      let p = ray.o + ray.d * t;
      emission + if let Some(obj) = obj {
        let color = obj.color.to_vec3(uv);
        match &obj.texture {
          Texture::Diffuse => {
            // random choose a point in unit circle using polar coordinate
            let r1 = 2.0 * PI * rng.gen();
            let r2 = rng.gen();
            let r2s = r2.sqrt();
            // w, u, v are unit vectors, perpendicular to each other
            // w is the norm vector against d
            let w = if norm.dot(ray.d) < 0.0 { norm } else { -norm };
            let u = w.get_orthogonal_for_unit();
            let v = w.cross(u);
            let d = (u * r1.cos() + v * r1.sin()) * r2s + w * (1.0 - r2).sqrt();
            color.schur(self.path_tracing_impl(Ray::new(p, d), dep + 1, rng))
          }
          Texture::Specular => {
            // reflect vector, also used in Refractive
            // doesn't need to call Ray::new, because I can guarantee |d|=1
            color.schur(self.path_tracing_impl(Ray { o: p, d: ray.d - norm * 2.0 * norm.dot(ray.d) }, dep + 1, rng))
          }
          Texture::Refractive => {
            let ((reflect, reflect_i), refract) = Texture::refract(p, norm, ray);
            if let Some((refract, refract_i)) = refract {
              color.schur(if dep >= 2 {
                if rng.gen() < reflect_i {
                  self.path_tracing_impl(reflect, dep + 1, rng)
                } else {
                  self.path_tracing_impl(refract, dep + 1, rng)
                }
              } else {
                self.path_tracing_impl(reflect, dep + 1, rng) * reflect_i
                  + self.path_tracing_impl(refract, dep + 1, rng) * refract_i
              })
            } else { self.path_tracing_impl(reflect, dep + 1, rng) }
          }
          _ => unimplemented!()
        }
      } else { self.env }
    } else { self.env }
  }
}
