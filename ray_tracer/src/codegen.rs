use super::world::*;
use super::geo::*;
use super::vec::*;
use super::material::*;
use super::mesh::*;
use std::mem;
use std::io::prelude::*;
use crate::kd_tree::*;
use crate::byteorder::*;
use std::fs::{File, remove_file};
use std::process::Command;
use crate::bezier::RotateBezier;
use crate::f128::f128;
use num_traits::cast::ToPrimitive;

fn cpp_vec3(v: Vec3) -> String {
  format!("Vec3{{{}, {}, {}}}", v.0, v.1, v.2)
}

pub trait BaseFn<Ch: BaseFn<Ch>> {
  fn gen_impl(this: &mut CodegenBase<Ch>, world: &World);

  fn gen_main(this: &mut CodegenBase<Ch>, world: &World);

  fn gen_trace_loop(this: &mut CodegenBase<Ch>, world: &World) {
    this.wln("for (u32 _ = 0; _ < 16; ++_) {").inc();
    this.wln("if (fac.len2() <= 1e-4) { return Vec3{}; }");
    this.wln("HitRes res{1e10};");
    for obj in &world.objs {
      Self::gen_geo(this, obj);
    }
    this.wln("{").inc();
    match &world.light.geo {
      LightGeo::Circle(circle) => {
        let plane = &circle.plane;
        this.wln(&format!("f32 dot_d_n = ray.d.dot({});", cpp_vec3(plane.n)));
        this.wln(&format!("f32 t = ({} - ray.o).dot({}) / dot_d_n;", cpp_vec3(plane.p), cpp_vec3(plane.n)));
        this.wln(&format!("if (t > EPS && t < res.t && (ray.o + ray.d * t - {}).len2() < {}) {{",
                          cpp_vec3(plane.p), circle.u.len2())).inc();
        this.wln(&format!("return fac.schur({});", cpp_vec3(world.light.emission)));
        this.dec().wln("}");
      }
    };
    this.dec().wln("}");
    Self::gen_handle_text(this);
    this.dec().wln("}");
  }

  fn gen_geo(this: &mut CodegenBase<Ch>, obj: &Object) {
    macro_rules! gen_color {
    ($data: ident, $w: ident, $h: ident, $image_handle: block) => {
        match &obj.color {
          Color::RGB(rgb) => { this.wln(&format!("res.col = {};", cpp_vec3(*rgb))); },
          Color::Image{data: $data, w: $w, h: $h} => $image_handle,
        };
      };
    }
    this.wln("{").inc();
    match &obj.geo {
      Geo::Sphere(sphere) => {
        this.wln(&format!("Vec3 oc = {} - ray.o;", cpp_vec3(sphere.c)));
        this.wln("f32 b = oc.dot(ray.d);");
        this.wln(&format!("f32 det = b * b - oc.len2() + {0} * {0};", sphere.r));
        this.wln("if (det > 0.0f) {").inc();
        this.wln("f32 sq_det = sqrtf(det);");
        this.wln("f32 t = b - sq_det > EPS ? b - sq_det : b + sq_det > EPS ? b + sq_det : 0.0f;");
        this.wln("if (t && t < res.t) {").inc();
        this.wln("res.t = t;");
        this.wln(&format!("res.norm = (ray.o + ray.d * t - {}).norm();", cpp_vec3(sphere.c)));
        this.wln(&format!("res.text = {};", Self::gen_text(obj.texture)));
        gen_color!(data, w, h, {
          this.wln("f32 u = 0.5f + atan2f(res.norm.z, res.norm.x) / (2.0f * PI);");
          this.wln("f32 v = 0.5f - asinf(res.norm.y) / PI;");
          Self::gen_img(this, data, *w, *h, false);
        });
        this.dec().wln("}").dec().wln("}");
      }
      Geo::InfPlane(plane) => {
        this.wln(&format!("f32 dot_d_n = ray.d.dot({});", cpp_vec3(plane.n)));
        this.wln(&format!("f32 t = ({} - ray.o).dot({}) / dot_d_n;", cpp_vec3(plane.p), cpp_vec3(plane.n)));
        this.wln("if (t > EPS && t < res.t) {").inc();
        this.wln("res.t = t;");
        this.wln(&format!("res.norm = {};", cpp_vec3(plane.n)));
        this.wln(&format!("res.text = {};", Self::gen_text(obj.texture)));
        gen_color!(data, w, h, {
            let u = plane.n.get_orthogonal_for_unit();
            let v = plane.n.cross(u);
            this.wln(&format!("Vec3 p = ray.o + ray.d * t - {};", cpp_vec3(plane.p)));
            // it is an arbitrary factor
            this.wln(&format!("f32 u = p.dot({});", cpp_vec3(u / 3.15)));
            this.wln(&format!("f32 v = p.dot({});", cpp_vec3(v / 3.15)));
            Self::gen_img(this, data, *w, *h, true);
          });
        this.dec().wln("}");
      }
      Geo::Circle(circle) => {
        let plane = &circle.plane;
        this.wln(&format!("f32 dot_d_n = ray.d.dot({});", cpp_vec3(plane.n)));
        this.wln(&format!("f32 t = ({} - ray.o).dot({}) / dot_d_n;", cpp_vec3(plane.p), cpp_vec3(plane.n)));
        this.wln(&format!("if (t > EPS && t < res.t && (ray.o + ray.d * t - {}).len2() < {}) {{",
                          cpp_vec3(plane.p), circle.u.len2())).inc();
        this.wln("res.t = t;");
        this.wln(&format!("res.norm = {};", cpp_vec3(plane.n)));
        this.wln(&format!("res.text = {};", Self::gen_text(obj.texture)));
        gen_color!(data, w, h, {
            let zero = circle.plane.p - circle.u - circle.v;
            this.wln(&format!("Vec3 p = ray.o + ray.d * t - {};", cpp_vec3(zero)));
            this.wln(&format!("f32 u = p.dot({});", cpp_vec3(circle.u * 0.5 / circle.u.len2())));
            this.wln(&format!("f32 v = p.dot({});", cpp_vec3(circle.v * 0.5 / circle.v.len2())));
            Self::gen_img(this, data, *w, *h, false);
          });
        this.dec().wln("}");
      }
      Geo::Rectangle(rectangle) => {
        let plane = &rectangle.plane;
        this.wln(&format!("f32 dot_d_n = ray.d.dot({});", cpp_vec3(plane.n)));
        this.wln(&format!("f32 t = ({} - ray.o).dot({}) / dot_d_n;", cpp_vec3(plane.p), cpp_vec3(plane.n)));
        this.wln("if (t > EPS && t < res.t) {").inc();
        this.wln(&format!("Vec3 p = ray.o + ray.d * t - {};", cpp_vec3(plane.p)));
        this.wln(&format!("f32 u = p.dot({});", cpp_vec3(rectangle.u * rectangle.inv_u_len)));
        this.wln(&format!("f32 v = p.dot({});", cpp_vec3(rectangle.v * rectangle.inv_v_len)));
        this.wln("if (0.0f < u && u < 1.0f && 0.0f < v && v < 1.0f) {").inc();
        this.wln("res.t = t;");
        this.wln(&format!("res.norm = {};", cpp_vec3(plane.n)));
        this.wln(&format!("res.text = {};", Self::gen_text(obj.texture)));
        gen_color!(data, w, h, { Self::gen_img(this, data, *w, *h, false); });
        this.dec().wln("}").dec().wln("}");
      }
      Geo::Mesh(mesh) => Self::gen_mesh(this, mesh, obj, None),
      Geo::RotateBezier(bezier) => {
        Self::gen_mesh(this, &bezier.mesh, obj, Some(bezier));
      }
    }
    this.dec().wln("}");
  }

  fn gen_mesh(this: &mut CodegenBase<Ch>, mesh: &Mesh, obj: &Object, bezier: Option<&RotateBezier>);

  // u & v should already be in scope, res.col should be set
  // need_warp: whether u & v need to be set between [0, 1)
  fn gen_img(this: &mut CodegenBase<Ch>, data: &[Vec3], w: u32, h: u32, need_warp: bool);

  fn gen_text(text: Texture) -> String {
    match text {
      Texture::Diffuse => "0".to_owned(),
      Texture::Specular => "1".to_owned(),
      Texture::Refractive => "2".to_owned(),
      Texture::Mixed { d_prob, s_prob } => format!("({{ f32 p = rng.gen(); p < {} ? 0 : p < {} ? 1 : 2; }})", d_prob, d_prob + s_prob),
    }
  }

  fn gen_handle_text(this: &mut CodegenBase<Ch>) {
    this.wln("if (res.t == 1e10) { break; }
    Vec3 p = ray.o + ray.d * res.t;
    fac = fac.schur(res.col);
    switch (res.text) {
      case 0: {
        f32 r1 = 2.0f * PI * rng.gen();
        f32 r2 = rng.gen(), r2s = sqrtf(r2);
        Vec3 w = res.norm.dot(ray.d) < 0.0f ? res.norm : -res.norm;
        Vec3 u = w.orthogonal_unit();
        Vec3 v = w.cross(u);
        Vec3 d = (u * cosf(r1) + v * sinf(r1)) * r2s + w * sqrtf(1.0f - r2);
        ray = {p, d.norm()};
        break;
      }
      case 1: {
        ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
        break;
      }
      case 2: {
        constexpr f32 NA = 1.0f, NG = 1.5f, R0 = (NA - NG) * (NA - NG) / ((NA + NG) * (NA + NG));
        f32 cos = res.norm.dot(ray.d), sin = sqrtf(1.0f - cos * cos), n;
        Vec3 norm_d = res.norm;
        if (cos < 0.0f) {
          n = NG / NA;
          cos = -cos;
          norm_d = -norm_d;
        } else {
          n = NA / NG;
          if (sin >= n) {
            ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
            break;
          }
        }
        if (rng.gen() < R0 + (1.0f - R0) * powf(1.0f - cos, 5)) {
          ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
        } else {
          ray = {p, norm_d * (sqrtf(1.0f - sin * sin / (n * n)) - cos / n) + ray.d / n};
        }
        break;
      }
    }");
  }
}

pub struct CodegenBase<Ch: BaseFn<Ch>> {
  ch: Ch,
  code: String,
  indent: String,
  // backward implementations
  impls: Vec<String>,
  mesh_id: u32,
  img_id: u32,
}

impl<Ch: BaseFn<Ch>> CodegenBase<Ch> {
  pub fn new(ch: Ch) -> Self {
    Self { ch, code: String::new(), indent: String::new(), impls: Vec::new(), mesh_id: 0, img_id: 0 }
  }

  pub fn gen(&mut self, world: &World, path: &str) {
    let mut header = File::open("tool/tracer_util.hpp").unwrap();
    let mut header_content = String::new();
    let _ = header.read_to_string(&mut header_content);
    self.wln(&header_content);
    Ch::gen_impl(self, world);
    self.wln("");
    Ch::gen_main(self, world);
    for impl_ in mem::replace(&mut self.impls, Vec::new()) {
      self.wln("");
      self.code += &impl_;
    }
    let _ = File::create(path).unwrap().write(self.code.as_bytes());
  }

  fn inc(&mut self) -> &mut Self {
    self.indent += "  ";
    self
  }

  fn dec(&mut self) -> &mut Self {
    self.indent.pop();
    self.indent.pop();
    self
  }

  fn wln(&mut self, s: &str) -> &mut Self {
    self.code += &self.indent;
    self.code += s;
    self.code += "\n";
    self
  }
}

fn gen_mesh_obj(id: u32, mesh: &Mesh, object: &Object) {
  let bin_path = format!("mesh{}", id);
  {
    let mut bin = File::create(&bin_path).unwrap();
    let mut data = Vec::new();
    fn walk(node: &KDNode, f: &mut Vec<u8>, mesh: &Mesh, object: &Object) -> usize {
      let ret = f.len(); // offset of self
      macro_rules! write_vec {
        ($vec: expr) => { let _ = (f.write_f32::<LittleEndian>($vec.0), f.write_f32::<LittleEndian>($vec.1), f.write_f32::<LittleEndian>($vec.2)); };
      }
      macro_rules! write_vec2 {
        ($vec: expr) => { let _ = (f.write_f32::<LittleEndian>($vec.0), f.write_f32::<LittleEndian>($vec.1)); };
      }
      write_vec!(node.aabb.min);
      write_vec!(node.aabb.max);
      match &node.kind {
        KDNodeKind::Internal(ch, sp_d, sp) => {
          f.write_u32::<LittleEndian>(0).unwrap();
          f.write_u32::<LittleEndian>(*sp_d).unwrap();
          f.write_f32::<LittleEndian>(*sp).unwrap();
          walk(&ch[0], f, mesh, object);
          let ch_off = walk(&ch[1], f, mesh, object) as u32;
          let ch_ptr = &mut f[ret + 24..ret + 28];
          ch_ptr[0] = (ch_off & 255) as u8;
          ch_ptr[1] = (ch_off >> 8 & 255) as u8;
          ch_ptr[2] = (ch_off >> 16 & 255) as u8;
          ch_ptr[3] = (ch_off >> 24 & 255) as u8;
        }
        // Fast Ray-Triangle Intersections by Coordinate Transformation
        // http://jcgt.org/published/0005/03/03/
        KDNodeKind::Leaf(idx) => {
          let len_pos = f.len();
          f.write_u32::<LittleEndian>(0).unwrap();
          let mut len = 0u32;
          let mut is_tri = vec![true; idx.len()];
          for (idx, &(i, j, k)) in idx.iter().enumerate() {
            let (p1, p2, p3) = (mesh.v[i as usize], mesh.v[j as usize], mesh.v[k as usize]);
            let (e1, e2) = (p2 - p1, p3 - p1);
            let mut m = [[0.0; 4]; 3];
            let norm = e1.cross(e2);
            if norm.0.abs() > norm.1.abs() && norm.0.abs() > norm.2.abs() {
              m[0][0] = 0.0;
              m[1][0] = 0.0;
              m[2][0] = 1.0;
              m[0][1] = e2.2 / norm.0;
              m[1][1] = -e1.2 / norm.0;
              m[2][1] = norm.1 / norm.0;
              m[0][2] = -e2.1 / norm.0;
              m[1][2] = e1.1 / norm.0;
              m[2][2] = norm.2 / norm.0;
              m[0][3] = p3.cross(p1).0 / norm.0;
              m[1][3] = -p2.cross(p1).0 / norm.0;
              m[2][3] = -p1.dot(norm) / norm.0;
            } else if norm.1.abs() > norm.2.abs() {
              m[0][0] = -e2.2 / norm.1;
              m[1][0] = e1.2 / norm.1;
              m[2][0] = norm.0 / norm.1;
              m[0][1] = 0.0;
              m[1][1] = 0.0;
              m[2][1] = 1.0;
              m[0][2] = e2.0 / norm.1;
              m[1][2] = -e1.0 / norm.1;
              m[2][2] = norm.2 / norm.1;
              m[0][3] = p3.cross(p1).1 / norm.1;
              m[1][3] = -p2.cross(p1).1 / norm.1;
              m[2][3] = -p1.dot(norm) / norm.1;
            } else if norm.2.abs() > 0.0 {
              m[0][0] = e2.1 / norm.2;
              m[1][0] = -e1.1 / norm.2;
              m[2][0] = norm.0 / norm.2;
              m[0][1] = -e2.0 / norm.2;
              m[1][1] = e1.0 / norm.2;
              m[2][1] = norm.1 / norm.2;
              m[0][2] = 0.0;
              m[1][2] = 0.0;
              m[2][2] = 1.0;
              m[0][3] = p3.cross(p1).2 / norm.2;
              m[1][3] = -p2.cross(p1).2 / norm.2;
              m[2][3] = -p1.dot(norm) / norm.2;
            } // else => degenerate triangle, all 0
            else {
              is_tri[idx] = false;
              continue;
            }
            len += 1;
            for row in &m {
              for &x in row {
                f.write_f32::<LittleEndian>(x).unwrap();
              }
            }
          }
          let len_ptr = &mut f[len_pos..len_pos + 4];
          let len = len | (1 << 31);
          len_ptr[0] = (len & 255) as u8;
          len_ptr[1] = (len >> 8 & 255) as u8;
          len_ptr[2] = (len >> 16 & 255) as u8;
          len_ptr[3] = (len >> 24 & 255) as u8;
          for (idx, &(i, j, k)) in idx.iter().enumerate() {
            if !is_tri[idx] { continue; }
            write_vec!(mesh.norm[i as usize]);
            write_vec!(mesh.norm[j as usize]);
            write_vec!(mesh.norm[k as usize]);
          }
          match &object.color {
            Color::Image { data: _, w: _, h: _ } => {
              for (idx, &(i, j, k)) in idx.iter().enumerate() {
                if !is_tri[idx] { continue; }
                write_vec2!(mesh.uv[i as usize]);
                write_vec2!(mesh.uv[j as usize]);
                write_vec2!(mesh.uv[k as usize]);
              }
            }
            _ => {}
          }
        }
      }
      ret
    }
    walk(&mesh.kd, &mut data, mesh, object);
    bin.write_all(&data).unwrap();
  }
  Command::new("ld").args(&["-r", "-b", "binary", &bin_path, "-o", &format!("mesh{}.o", id)]).spawn().unwrap().wait().unwrap();
  remove_file(&bin_path).unwrap();
}

// img is accessed though float4 on gpu
fn gen_img_obj(id: u32, data: &[Vec3], as_float4: bool) {
  let bin_path = format!("img{}", id);
  {
    let mut bin = File::create(&bin_path).unwrap();
    let mut f = Vec::with_capacity(data.len() * 12);
    for v in data {
      f.write_f32::<LittleEndian>(v.0).unwrap();
      f.write_f32::<LittleEndian>(v.1).unwrap();
      f.write_f32::<LittleEndian>(v.2).unwrap();
      if as_float4 {
        f.write_f32::<LittleEndian>(0.0).unwrap();
      }
    }
    bin.write_all(&f).unwrap();
  }
  Command::new("ld").args(&["-r", "-b", "binary", &bin_path, "-o", &format!("img{}.o", id)]).spawn().unwrap().wait().unwrap();
  remove_file(&bin_path).unwrap();
}

pub struct CppCodegen;

impl BaseFn<CppCodegen> for CppCodegen {
  fn gen_impl(this: &mut CodegenBase<CppCodegen>, world: &World) {
    this.wln("Vec3 trace(Ray ray, XorShiftRNG &rng) {").inc();
    this.wln("Vec3 fac{1.0f, 1.0f, 1.0f};");
    Self::gen_trace_loop(this, world);
    this.wln("return Vec3{};");
    this.dec().wln("}");
  }

  fn gen_main(this: &mut CodegenBase<CppCodegen>, world: &World) {
    this.wln(&format!(r#"const u32 W = {}, H = {};"#, world.w, world.h));
    this.wln(r#"Vec3 output[W * H];

int main(int argc, char **args) {
  u32 ns = argc > 1 ? std::atoi(args[1]) : (puts("please specify #sample"), exit(-1), 0);"#).inc();
    let cx = Vec3(world.w as f32 * 0.5135 / world.h as f32, 0.0, 0.0);
    let cy = cx.cross(world.cam.d).norm() * 0.5135;
    this.wln(&format!("constexpr Ray cam{{{}, {}}};", cpp_vec3(world.cam.o), cpp_vec3(world.cam.d)));
    this.wln(&format!("constexpr Vec3 cx{{{}, {}, {}}};", cx.0, cx.1, cx.2));
    this.wln(&format!("constexpr Vec3 cy{{{}, {}, {}}};", cy.0, cy.1, cy.2)).dec();
    this.wln(r#"#pragma omp parallel for schedule(dynamic, 1)
  for (u32 y = 0; y < H; ++y) {
    fprintf(stderr, "\rrendering %5.2f%%", 100.0f * y / (H - 1));
    for (u32 x = 0; x < W; ++x) {
      u32 index = y * W + x;
      Vec3 sum{};
      XorShiftRNG rng{index};
      for (u32 s = 0; s < ns / 4; ++s) {
        for (u32 sx = 0; sx < 2; ++sx) {
          for (u32 sy = 0; sy < 2; ++sy) {
            f32 r1 = 2.0f * rng.gen(), r2 = 2.0f * rng.gen();
            f32 dx = r1 < 1.0f ? sqrtf(r1) - 1.0f : 1.0f - sqrtf(2.0f - r1);
            f32 dy = r2 < 1.0f ? sqrtf(r2) - 1.0f : 1.0f - sqrtf(2.0f - r2);
            Vec3 d = cx * (((sx + 0.5f + dx) * 0.5f + x) / W - 0.5f) +
                     cy * (((sy + 0.5f + dy) * 0.5f + y) / H - 0.5f) + cam.d;
            sum += trace(Ray{cam.o + d * 14.0f, d.norm()}, rng);
          }
        }
      }
      output[index] = sum / ns;
    }
  }
  output_png(output, W, H, argc > 2 ? args[2] : "image.png");
}"#);
  }

  fn gen_mesh(this: &mut CodegenBase<CppCodegen>, mesh: &Mesh, obj: &Object, bezier: Option<&RotateBezier>) {
    let id = this.mesh_id;
    this.mesh_id += 1;
    this.wln(&format!("extern const KDNode _binary_mesh{}_start;", id));
    if let Some(bezier) = bezier {
      fn gen_coef(this: &mut CodegenBase<CppCodegen>, ps: &[F64Vec3], name: &str) {
        let n = ps.len() - 1;
        let mut cs = vec![vec![0; n + 1]; n + 1];
        let mut coef = vec![(f128::new(0.0), f128::new(0.0)); n + 1];
        cs[0][0] = 1;
        for i in 1..=n {
          cs[i][0] = 1;
          for j in 1..=i {
            cs[i][j] = cs[i - 1][j] + cs[i - 1][j - 1];
          }
        }
        for i in 0..=n {
          let fac_x = f128::new(ps[i].0) * f128::new(cs[n][i]);
          let fac_y = f128::new(ps[i].1) * f128::new(cs[n][i]);
          for j in i..=n {
            let tmp = fac_x * f128::new(cs[n - i][j - i]);
            coef[j].0 += if (j - i) % 2 == 1 { -tmp } else { tmp };
            let tmp = fac_y * f128::new(cs[n - i][j - i]);
            coef[j].1 += if (j - i) % 2 == 1 { -tmp } else { tmp };
          }
        }
        let mut data = String::new();
        // rev for convenient in C++
        for (x, y) in coef.iter().rev() {
          data += &format!("{{{}, {}}}, ", x.to_f32().unwrap(), y.to_f32().unwrap());
        }
        this.wln(&format!("constexpr f32 {}[][2] = {{{}}};", name, data));
      }
      gen_coef(this, &bezier.curve.ps, "PS");
      gen_coef(this, &bezier.curve.der_ps, "DER");
      this.wln(&format!("constexpr f32 SHIFT_X = {}, SHIFT_Z = {};", bezier.shift_x, bezier.shift_z));
      this.wln(&format!("if (kd_node_hit(&_binary_mesh{}_start, ray, res, {}, {})) {{", id, Self::gen_text(obj.texture), cpp_vec3(Vec3(-1.0, 0.0, 0.0)))).inc();
      this.wln("f32 u = res.col.x * (2 * PI);
        f32 v = res.col.y;
        f32 t = res.t;
        f32 bx, by, dbx, dby;
        Vec3 o{ray.o.x - SHIFT_X, ray.o.y, ray.o.z - SHIFT_Z};
        Vec3 d = ray.d;
        f32 a00, a01, a02, b0;
        f32      a11, a12, b1;
        f32 a20, a21, a22, b2;
        f32 err;
        for (u32 i = 0; i < 5; ++i) {
          EVAL_BEZIER(PS, v, bx, by);
          EVAL_BEZIER(DER, v, dbx, dby);
          a00 = -bx * sinf(u), a01 = dbx * cosf(u) , a02 = -d.x, b0 = bx * cosf(u) - o.x - t * d.x;
          /* a10 = 0       ,*/ a11 = dby           , a12 = -d.y, b1 = by - o.y - t * d.y;
          a20 = -bx * cosf(u), a21 = -dbx * sinf(u), a22 = -d.z, b2 = -bx * sinf(u)- o.z - t * d.z;
          err = b0 * b0 + b1 * b1 + b2 * b2;
          {
            f32 fac = a20 / a00;
            a21 -= fac * a01, a22 -= fac * a02, b2 -= fac * b0;
          }
          {
            f32 fac = a21 / a11;
            a22 -= fac * a12, b2 -= fac * b1;
          }
          f32 x2 = b2 / a22;
          f32 x1 = (b1 - x2 * a12) / a11;
          f32 x0 = (b0 - x2 * a02 - x1 * a01) / a00;
          u -= x0, v -= x1, t -= x2;
        }");
      this.wln("if (err < 0.01) {").inc();
      this.wln("res.t = t;");
      this.wln("res.norm = Vec3{a01, a11, a21}.cross(Vec3{a00, 0, a20}).norm();");
      match &obj.color {
        Color::Image { data, w, h } => {
          this.wln("u = fmaxf(fminf(u / (2 * PI), 0.9999), 0);");
          this.wln("v = fmaxf(fminf(v, 0.9999), 0);");
          Self::gen_img(this, data, *w, *h, false);
        }
        Color::RGB(rgb) => { this.wln(&format!("res.col = {};", cpp_vec3(*rgb))); }
      };
      this.dec().wln("}");
      this.dec().wln("}");
    } else {
      match &obj.color {
        Color::Image { data, w, h } => {
          this.wln(&format!("if (kd_node_hit(&_binary_mesh{}_start, ray, res, {}, {})) {{", id, Self::gen_text(obj.texture), cpp_vec3(Vec3(-1.0, 0.0, 0.0)))).inc();
          this.wln("f32 u = res.col.x;");
          this.wln("f32 v = res.col.y;");
          Self::gen_img(this, data, *w, *h, false);
          this.dec().wln("}");
        }
        Color::RGB(rgb) => {
          this.wln(&format!("kd_node_hit(&_binary_mesh{}_start, ray, res, {}, {});", id, Self::gen_text(obj.texture), cpp_vec3(*rgb)));
        }
      };
    }
    gen_mesh_obj(id, mesh, obj);
  }

  fn gen_img(this: &mut CodegenBase<CppCodegen>, data: &[Vec3], w: u32, h: u32, need_warp: bool) {
    let id = this.img_id;
    this.img_id += 1;
    this.wln(&format!("extern const Vec3 _binary_img{}_start[];", id));
    if need_warp {
      this.wln("u = mod1(u);");
      this.wln("v = mod1(v);");
    }
    this.wln(&format!("res.col = _binary_img{}_start[u32(v * {h}) * {w} + u32(u * {w})];", id, h = h, w = w));
    gen_img_obj(id, data, false);
  }
}

pub struct CudaCodegen {
  img_wh: Vec<(u32, u32)>,
}

impl CudaCodegen {
  pub fn new() -> CudaCodegen {
    CudaCodegen { img_wh: Vec::new() }
  }
}

impl BaseFn<CudaCodegen> for CudaCodegen {
  fn gen_impl(this: &mut CodegenBase<CudaCodegen>, world: &World) {
    this.wln("DEVICE Vec3 trace_impl(Ray ray, XorShiftRNG &rng) {").inc();
    this.wln("Vec3 fac{1.0f, 1.0f, 1.0f};");
    Self::gen_trace_loop(this, world);
    this.wln("return Vec3{};");
    this.dec().wln("}\n");

    this.wln("GLOBAL void trace(Vec3 *gpu_output, u32 ns) {").inc();
    let cx = Vec3(world.w as f32 * 0.5135 / world.h as f32, 0.0, 0.0);
    let cy = cx.cross(world.cam.d).norm() * 0.5135;
    this.wln(&format!("constexpr Ray cam{{{}, {}}};", cpp_vec3(world.cam.o), cpp_vec3(world.cam.d)));
    this.wln(&format!("constexpr Vec3 cx{{{}, {}, {}}};", cx.0, cx.1, cx.2));
    this.wln(&format!("constexpr Vec3 cy{{{}, {}, {}}};", cy.0, cy.1, cy.2)).dec();
    this.wln(&format!(r#"
  u32 x = blockIdx.x, y = blockIdx.y, th = threadIdx.x;
  u32 index = y * {w} + x;
  XorShiftRNG rng{{index + th * 19260817}};
  Vec3 sum{{}};
  for (u32 s = 0; s < ns / (CUDA_BLOCK_SIZE * 4); ++s) {{
    for (u32 sx = 0; sx < 2; ++sx) {{
      for (u32 sy = 0; sy < 2; ++sy) {{
        f32 r1 = 2.0f * rng.gen(), r2 = 2.0f * rng.gen();
        f32 dx = r1 < 1.0f ? sqrtf(r1) - 1.0f : 1.0f - sqrtf(2.0f - r1);
        f32 dy = r2 < 1.0f ? sqrtf(r2) - 1.0f : 1.0f - sqrtf(2.0f - r2);
        Vec3 d = cx * (((sx + 0.5f + dx) * 0.5f + x) / {w} - 0.5f) + cy * (((sy + 0.5f + dy) * 0.5f + y) / {h} - 0.5f) + cam.d;
        sum += trace_impl(Ray{{cam.o, d.norm()}}, rng);
      }}
    }}
  }}
  sum /= ns;
  atomicAdd(&gpu_output[index].x, sum.x);
  atomicAdd(&gpu_output[index].y, sum.y);
  atomicAdd(&gpu_output[index].z, sum.z);
}}"#, w = world.w, h = world.h));
  }

  fn gen_main(this: &mut CodegenBase<CudaCodegen>, world: &World) {
    this.wln(&format!("Vec3 cpu_output[{}];", world.w * world.h));
    for i in 0..this.mesh_id {
      this.wln(&format!("CONSTANT const KDNode * __restrict__ gpu_mesh{};", i));
    }
    for i in 0..this.ch.img_wh.len() {
      this.wln(&format!("texture<float4, 2, cudaReadModeElementType> gpu_img{};", i));
      this.wln(&format!("cudaArray *gpu_img{}_arr;", i));
    }
    this.wln("");
    this.wln("void init_res() {").inc();
    for i in 0..this.mesh_id {
      this.wln(&format!(r#"extern const u8 _binary_mesh{i}_start;
  extern const u8 _binary_mesh{i}_end;
  KDNode *gpu_mesh{i}_tmp;
  cudaMalloc(&gpu_mesh{i}_tmp, &_binary_mesh{i}_end - &_binary_mesh{i}_start);
  cudaMemcpy(gpu_mesh{i}_tmp, &_binary_mesh{i}_start, &_binary_mesh{i}_end - &_binary_mesh{i}_start, cudaMemcpyHostToDevice);
  cudaMemcpyToSymbol(gpu_mesh{}, &gpu_mesh{i}_tmp, sizeof(KDNode *));"#, i = i));
    }
    for (i, (w, h)) in this.ch.img_wh.clone().iter().enumerate() {
      this.wln(&format!(r#"extern const u8 _binary_img{i}_start;
  extern const u8 _binary_img{i}_end;
  cudaChannelFormatDesc desc{i} = cudaCreateChannelDesc<float4>();
  cudaMallocArray(&gpu_img{i}_arr, &desc{i}, {w}, {h});
  cudaMemcpyToArray(gpu_img{i}_arr, 0, 0, &_binary_img{i}_start, &_binary_img{i}_end - &_binary_img{i}_start, cudaMemcpyHostToDevice);
  gpu_img{i}.addressMode[0] = cudaAddressModeWrap;
  gpu_img{i}.addressMode[1] = cudaAddressModeWrap;
  gpu_img{i}.filterMode = cudaFilterModeLinear;
  gpu_img{i}.normalized = true;
  cudaBindTextureToArray(gpu_img{i}, gpu_img{i}_arr, desc{i});
  "#, i = i, w = *w, h = *h));
    }
    this.dec().wln("}\n");
    this.wln("void free_res() {").inc();
    for i in 0..this.mesh_id {
      this.wln(&format!(r#"KDNode *gpu_mesh{i}_tmp;
  cudaMemcpyFromSymbol(&gpu_mesh{i}_tmp, gpu_mesh{}, sizeof(KDNode *));
  cudaFree(gpu_mesh{i}_tmp);"#, i = i));
    }
    for i in 0..this.ch.img_wh.len() {
      this.wln(&format!("cudaFreeArray(gpu_img{}_arr);", i));
    }
    this.dec().wln("}\n");
    this.wln(&format!(r#"int main(int argc, char **args) {{
  u32 ns = argc > 1 ? atoi(args[1]) : (puts("please specify #sample"), exit(-1), 0);
  ns = (ns + (CUDA_BLOCK_SIZE * 4) - 1) / (CUDA_BLOCK_SIZE * 4) * (CUDA_BLOCK_SIZE * 4);
  init_res();
  Vec3 *gpu_output;
  cudaMalloc(&gpu_output, {wh} * sizeof(Vec3));
  trace<<<dim3({}, {}, 1), CUDA_BLOCK_SIZE>>>(gpu_output, ns);
  cudaMemcpy(cpu_output, gpu_output, {wh} * sizeof(Vec3), cudaMemcpyDeviceToHost);
  cudaFree(gpu_output);
  free_res();
  CUDA_CHECK_ERROR(cudaGetLastError());
  output_png(cpu_output, {w}, {h}, argc > 2 ? args[2] : "image.png");
}}"#, w = world.w, h = world.h, wh = world.w * world.h));
  }

  fn gen_mesh(this: &mut CodegenBase<CudaCodegen>, mesh: &Mesh, obj: &Object, _bezier: Option<&RotateBezier>) {
    let id = this.mesh_id;
    this.mesh_id += 1;
    this.wln(&format!("extern CONSTANT const KDNode * __restrict__ gpu_mesh{};", id));
    gen_mesh_obj(id, mesh, obj);
    {}
    match &obj.color {
      Color::Image { data, w, h } => {
        this.wln(&format!("if (kd_node_hit(gpu_mesh{}, ray, res, {}, {})) {{", id, Self::gen_text(obj.texture), cpp_vec3(Vec3(-1.0, 0.0, 0.0)))).inc();
        this.wln("f32 u = res.col.x;");
        this.wln("f32 v = res.col.y;");
        Self::gen_img(this, data, *w, *h, false);
        this.dec().wln("}");
      }
      Color::RGB(rgb) => {
        this.wln(&format!("kd_node_hit(gpu_mesh{}, ray, res, {}, {});", id, Self::gen_text(obj.texture), cpp_vec3(*rgb)));
      }
    };
  }

  fn gen_img(this: &mut CodegenBase<CudaCodegen>, data: &[Vec3], w: u32, h: u32, _need_warp: bool) {
    let id = this.ch.img_wh.len() as u32;
    this.ch.img_wh.push((w, h));
    this.wln(&format!("extern texture<float4, 2, cudaReadModeElementType> gpu_img{};", id));
    this.wln(&format!("res.col = Vec3::from_float4(tex2D(gpu_img{}, u, v));", id));
    gen_img_obj(id, data, true);
  }
}

pub struct PPMCodeGen {
  // which pass am I generating
  pass: u32,
  // my algorithm still have some defects due to precision
  // please user specify the bound of the picture
  min: Vec3,
  max: Vec3,
}

impl PPMCodeGen {
  pub fn new(min: Vec3, max: Vec3) -> PPMCodeGen {
    PPMCodeGen { pass: 0, min, max }
  }
}

impl BaseFn<PPMCodeGen> for PPMCodeGen {
  fn gen_impl(this: &mut CodegenBase<PPMCodeGen>, world: &World) {
    let mut header = File::open("tool/ppm_util.hpp").unwrap();
    let mut header_content = String::new();
    let _ = header.read_to_string(&mut header_content);
    this.wln(&header_content);
    this.wln("void hit_point_pass(Ray ray, Vec3 fac, u32 dep, u32 index) {").inc();
    this.wln("for (; dep < 20; ++dep) {").inc();
    this.wln("HitRes res{1e10};");
    for obj in &world.objs {
      Self::gen_geo(this, obj);
    }
    this.wln(&format!(r#"if (res.t == 1e10) {{ break; }}
    Vec3 p = ray.o + ray.d * res.t;
    if (p.x < {} - EPS || p.y < {} - EPS || p.z < {} - EPS || p.x > {} + EPS || p.y > {} + EPS || p.z > {} + EPS) {{ return; }}
    "#, this.ch.min.0, this.ch.min.1, this.ch.min.2, this.ch.max.0, this.ch.max.1, this.ch.max.2));
    this.wln(r"fac = fac.schur(res.col);
    switch (res.text) {
      case 0: {
        grid.mu.lock();
        grid.hps.push_back(HitPoint{.fac=fac, .pos=p, .norm=res.norm, .flux=Vec3{}, .r2=0, .n=0, .idx = index});
        grid.mu.unlock();
        return;
      }
      case 1: {
        ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
        break;
      }
      case 2: {
        constexpr f32 NA = 1.0f, NG = 1.5f, R0 = (NA - NG) * (NA - NG) / ((NA + NG) * (NA + NG));
        f32 cos = res.norm.dot(ray.d), sin = sqrtf(1.0f - cos * cos), n;
        Vec3 norm_d = res.norm;
        if (cos < 0.0f) {
          n = NG / NA;
          cos = -cos;
          norm_d = -norm_d;
        } else {
          n = NA / NG;
          if (sin >= n) {
            ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
            break;
          }
        }
        f32 prob = R0 + (1.0f - R0) * powf(1.0f - cos, 5);
        hit_point_pass({p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)}, fac * prob, dep + 1, index);
        ray = {p, norm_d * (sqrtf(1.0f - sin * sin / (n * n)) - cos / n) + ray.d / n};
        fac *= (1 - prob);
        break;
      }
    }");
    this.dec().wln("}").dec().wln("}\n");
    this.ch.pass = 1;
    this.img_id = 0;
    this.mesh_id = 0;
    this.wln("void photon_pass(Ray ray, Vec3 flux, u32 seed) {").inc();
    this.wln("Vec3 fac{1.0f, 1.0f, 1.0f};");
    this.wln("for (u32 d = 0; d < 20; ++d) {").inc();
    this.wln("HitRes res{1e10};");
    for obj in &world.objs {
      Self::gen_geo(this, obj);
    }
    this.wln(&format!(r#"if (res.t == 1e10) {{ break; }}
    Vec3 p = ray.o + ray.d * res.t;
    if (p.x < {} - EPS || p.y < {} - EPS || p.z < {} - EPS || p.x > {} + EPS || p.y > {} + EPS || p.z > {} + EPS) {{ return; }}
    u32 d3 = d * 3 + 5;"#, this.ch.min.0, this.ch.min.1, this.ch.min.2, this.ch.max.0, this.ch.max.1, this.ch.max.2));
    this.wln(r#"switch (res.text) {
      case 0: {
        Vec3 pos = (p - grid.min) * grid.inv_grid_size;
        u32 h = grid.hash(u32(pos.x), u32(pos.y), u32(pos.z));
        for (u32 i = grid.idx[h], end = grid.idx[h + 1]; i < end; ++i) {
          HitPoint *hp = grid.pool[i];
          Vec3 v = hp->pos - p;
          if (hp->norm.dot(res.norm) > EPS && v.dot(v) <= hp->r2) {
            f32 g = (hp->n * ALPHA + ALPHA) / (hp->n * ALPHA + 1.0);
            hp->r2 = hp->r2 * g;
            ++hp->n;
            hp->flux = (hp->flux + hp->fac.schur(flux) * (1 / PI)) * g;
          }
        }
        f32 prob = res.col.x > res.col.y && res.col.x > res.col.z ? res.col.x : res.col.y > res.col.z ? res.col.y : res.col.z;
        if (hal[d3 + 1].gen(seed) < prob) {
          flux = flux.schur(res.col) / prob;
          f32 r1 = 2 * PI * hal[d3 - 1].gen(seed), r2 = hal[d3].gen(seed), r2s = sqrtf(r2);
          Vec3 w = res.norm.dot(ray.d) < 0.0f ? res.norm : -res.norm;
          Vec3 u = w.orthogonal_unit();
          Vec3 v = w.cross(u);
          Vec3 d = (u * cosf(r1) + v * sinf(r1)) * r2s + w * sqrtf(1.0f - r2);
          ray = {p, d.norm()};
        } else {
          return;
        }
        break;
      }
      case 1: {
        fac = fac.schur(res.col);
        ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
        break;
      }
      case 2: {
        fac = fac.schur(res.col);
        constexpr f32 NA = 1.0f, NG = 1.5f, R0 = (NA - NG) * (NA - NG) / ((NA + NG) * (NA + NG));
        f32 cos = res.norm.dot(ray.d), sin = sqrtf(1.0f - cos * cos), n;
        Vec3 norm_d = res.norm;
        if (cos < 0.0f) {
          n = NG / NA;
          cos = -cos;
          norm_d = -norm_d;
        } else {
          n = NA / NG;
          if (sin >= n) {
            ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
            break;
          }
        }
        if (hal[d3 - 1].gen(seed) < R0 + (1.0f - R0) * powf(1.0f - cos, 5)) {
          ray = {p, ray.d - res.norm * 2.0f * res.norm.dot(ray.d)};
        } else {
          ray = {p, norm_d * (sqrtf(1.0f - sin * sin / (n * n)) - cos / n) + ray.d / n};
        }
        break;
      }
    }"#);
    this.dec().wln("}").dec().wln("}\n");
  }

  fn gen_main(this: &mut CodegenBase<PPMCodeGen>, world: &World) {
    this.wln(&format!(r#"const u32 W = {}, H = {};"#, world.w, world.h));
    this.wln(r#"Vec3 output[W * H];

int main(int argc, char **args) {
  u32 np_1024 = argc > 1 ? std::atoi(args[1]) : (puts("please specify (#photon / 1024)"), exit(-1), 0);"#).inc();
    let cx = Vec3(world.w as f32 * 0.5135 / world.h as f32, 0.0, 0.0);
    let cy = cx.cross(world.cam.d).norm() * 0.5135;
    this.wln(&format!("constexpr Ray cam{{{}, {}}};", cpp_vec3(world.cam.o), cpp_vec3(world.cam.d)));
    this.wln(&format!("constexpr Vec3 cx{{{}, {}, {}}};", cx.0, cx.1, cx.2));
    this.wln(&format!("constexpr Vec3 cy{{{}, {}, {}}};", cy.0, cy.1, cy.2)).dec();
    this.wln(r#"#pragma omp parallel for schedule(dynamic, 1)
  for (u32 y = 0; y < H; ++y) {
    fprintf(stdout, "\rrendering: hit point pass %5.2f%%", 100.0 * y / (H - 1));
    for (u32 x = 0; x < W; ++x) {
      u32 index = y * W + x;
      for (u32 sx = 0; sx < 2; ++sx) {
        for (u32 sy = 0; sy < 2; ++sy) {
          f32 r1 = 2.0f * hal[sx * 4 + sy * 2].gen(index), r2 = 2.0f * hal[sx * 4 + sy * 2 + 1].gen(index);
          f32 dx = r1 < 1.0f ? sqrtf(r1) - 1.0f : 1.0f - sqrtf(2.0f - r1);
          f32 dy = r2 < 1.0f ? sqrtf(r2) - 1.0f : 1.0f - sqrtf(2.0f - r2);
          Vec3 d = cx * (((sx + 0.5f + dx) * 0.5f + x) / W - 0.5f)
                   + cy * (((sy + 0.5f + dy) * 0.5f + y) / H - 0.5f) + cam.d;
          hit_point_pass(Ray{cam.o + d * 14.0f, d.norm()}, Vec3{0.25, 0.25, 0.25}, 0, index);
        }
      }
    }
  }
  fprintf(stderr, "\n");
  grid.rebuild(W, H);"#);
    this.wln("#pragma omp parallel for schedule(dynamic, 1)").inc();
    this.wln("for (u32 i = 0; i < np_1024; ++i) {").inc();
    this.wln(r#"fprintf(stdout, "\rrendering: photon pass %5.2f%%", 100.0f * i / (np_1024 - 1));"#);
    this.wln("u32 base = i * 1024;");
    this.wln("for (u32 j = 0; j < 1024; j++) {").inc();
    match &world.light.geo {
      LightGeo::Circle(circle) => {
        let plane = &circle.plane;
        this.wln("f32 th1 = 2 * PI * hal[0].gen(base + j), r = sqrtf(hal[1].gen(base + j));");
        this.wln("f32 th2 = 2 * PI * hal[2].gen(base + j), th3 = 2 * acosf(sqrtf(1 - hal[3].gen(base + j)));");
        this.wln(&format!("Ray ray{{{} + {} * r * cosf(th1) + {} * r * sinf(th1), Vec3{{cosf(th2) * sinf(th3), cosf(th3), sinf(th2) * sinf(th3)}}}};",
                          cpp_vec3(plane.p), cpp_vec3(circle.u), cpp_vec3(circle.v)));
        this.wln("photon_pass(ray, Vec3{25, 25, 25} * (PI * 4.0), base + j);");
      }
    };
    this.dec().wln("}").dec().wln("}");
    this.wln(r#"for (auto &hp : grid.hps) {
    output[hp.idx] += hp.flux * (1.0f / (PI * hp.r2 * np_1024 * 1000.0f));
  }
  output_png(output, W, H, argc > 2 ? args[2] : "image.png");
  fprintf(stdout, "\n");
}
"#);
  }

  // copied CppCodeGen::gen_mesh
  fn gen_mesh(this: &mut CodegenBase<PPMCodeGen>, mesh: &Mesh, obj: &Object, _bezier: Option<&RotateBezier>) {
    let id = this.mesh_id;
    this.mesh_id += 1;
    this.wln(&format!("extern const KDNode _binary_mesh{}_start;", id));
    if this.ch.pass == 0 {
      gen_mesh_obj(id, mesh, obj);
    }
    match &obj.color {
      Color::Image { data, w, h } => {
        this.wln(&format!("if (kd_node_hit(&_binary_mesh{}_start, ray, res, {}, {})) {{", id, Self::gen_text(obj.texture), cpp_vec3(Vec3(-1.0, 0.0, 0.0)))).inc();
        this.wln("f32 u = res.col.x;");
        this.wln("f32 v = res.col.y;");
        Self::gen_img(this, data, *w, *h, false);
        this.dec().wln("}");
      }
      Color::RGB(rgb) => {
        this.wln(&format!("kd_node_hit(&_binary_mesh{}_start, ray, res, {}, {});", id, Self::gen_text(obj.texture), cpp_vec3(*rgb)));
      }
    };
  }

  // copied CppCodeGen::gen_img
  fn gen_img(this: &mut CodegenBase<PPMCodeGen>, data: &[Vec3], w: u32, h: u32, need_warp: bool) {
    let id = this.img_id;
    this.img_id += 1;
    this.wln(&format!("extern const Vec3 _binary_img{}_start[];", id));
    if need_warp {
      this.wln("u = mod1(u);");
      this.wln("v = mod1(v);");
    }
    this.wln(&format!("res.col = _binary_img{}_start[u32(v * {h}) * {w} + u32(u * {w})];", id, h = h, w = w));
    if this.ch.pass == 0 {
      gen_img_obj(id, data, false);
    }
  }
}