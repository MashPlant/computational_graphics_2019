use super::vec::*;
use super::geo::Ray;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum Texture {
  Diffuse,
  Specular,
  Refractive,
  Mixed { d_prob: f32, s_prob: f32 },
}

impl Texture {
  // return ((reflect ray, reflect intensity), (refract ray, refract intensity))
  // if total reflect happens, refract intensity is 0, refract ray has no meaning
  pub fn refract(p: Vec3, norm: Vec3, ray: Ray) -> ((Ray, f32), Option<(Ray, f32)>) {
    const NA: f32 = 1.0; // refractive index of air
    const NG: f32 = 1.5; // refractive index of glass
    // R(theta) = R(0)+(1-R(0))*(1-cos(theta))^5, determine the identity of reflect light
    const R0: f32 = (NA - NG) * (NA - NG) / ((NA + NG) * (NA + NG));
    let reflect = Ray { o: p, d: ray.d - norm * 2.0 * norm.dot(ray.d) };
    let mut cos_theta = norm.dot(ray.d);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let mut norm_d = norm; // -norm or norm, so that norm_d.dot(d) > 0
    let n;
    if cos_theta < 0.0 { // in
      n = NG / NA;
      cos_theta = -cos_theta;
      norm_d = -norm_d;
    } else { // out
      n = NA / NG;
      if sin_theta >= n { // total reflection
        return ((reflect, 1.0), None);
      }
    }
    // the vector of refract light, get it by solving |an+bd|=1, sin(<an+bd,n>)=sin(theta)/n
    // doesn't need to call Ray::new, because I can guarantee |d|=1
    let refract = Ray { o: p, d: norm_d * ((1.0 - sin_theta * sin_theta / (n * n)).sqrt() - cos_theta / n) + ray.d / n };
    let reflect_i = R0 + (1.0 - R0) * (1.0 - cos_theta).powi(5);
    let refract_i = 1.0 - reflect_i;
    ((reflect, reflect_i), Some((refract, refract_i)))
  }
}

#[derive(Serialize, Deserialize)]
pub enum Color {
  RGB(Vec3),
  Image { data: Box<[Vec3]>, w: u32, h: u32 },
}

impl Color {
  pub fn to_vec3(&self, uv: Vec2) -> Vec3 {
    match self {
      Color::RGB(rgb) => *rgb,
      &Color::Image { ref data, w, h } => {
        let y = (uv.1 * h as f32) as u32 % h;
        let x = (uv.0 * w as f32) as u32 % w;
        data[((y * w + x) as usize)]
      }
    }
  }
}