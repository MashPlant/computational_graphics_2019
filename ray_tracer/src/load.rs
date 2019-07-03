use super::vec::*;
use super::mesh::Mesh;
use super::bezier::*;
use super::mat44::Mat44;
use super::material::Color;
use std::fs::*;
use std::io::prelude::*;
use std::io;
use std::collections::HashMap;

pub fn mesh(path: &str, transform: Mat44) -> io::Result<Mesh> {
  let (mut tmp_v, mut tmp_uv, mut tmp_norm, mut tmp_index) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
  tmp_uv.push(Vec2(0.5, 0.5)); // dummy uv
  let file = File::open(path)?;
  for line in io::BufReader::new(file).lines() {
    let line = line?;
    let (x, y, z): (f32, f32, f32);
    if line.starts_with("vn") {
      scan!(line.bytes() => "vn {} {} {}", x, y, z);
      tmp_norm.push((transform * Vec4(x, y, z, 0.0)).to_vec3().norm());
    } else if line.starts_with("vt") {
      scan!(line.bytes() => "vt {} {}", x, y);
      tmp_uv.push(Vec2(x, -y));
    } else if line.starts_with("v") {
      scan!(line.bytes() => "v {} {} {}", x, y, z);
      tmp_v.push((transform * Vec4(x, y, z, 1.0)).to_vec3());
    } else if line.starts_with("f") {
      let rem = line.split_whitespace().skip(1).collect::<Vec<_>>();
      assert!(rem.len() >= 3);
      let v1 = rem[0].split('/').collect::<Vec<_>>();
      assert_eq!(v1.len(), 3);
      for (v2, v3) in rem.iter().skip(1).zip(rem.iter().skip(2)) {
        let v2 = v2.split('/').collect::<Vec<_>>();
        let v3 = v3.split('/').collect::<Vec<_>>();
        assert_eq!(v2.len(), 3);
        assert_eq!(v3.len(), 3);
        tmp_index.push(((v1[0].parse::<u32>().unwrap() - 1, v1[1].parse::<u32>().unwrap_or(0), v1[2].parse::<u32>().unwrap() - 1),
                        (v2[0].parse::<u32>().unwrap() - 1, v2[1].parse::<u32>().unwrap_or(0), v2[2].parse::<u32>().unwrap() - 1),
                        (v3[0].parse::<u32>().unwrap() - 1, v3[1].parse::<u32>().unwrap_or(0), v3[2].parse::<u32>().unwrap() - 1)));
      }
//      match rem.len() {
//        3 => {}
//        4 => {}
//        _ => panic!("not a mesh"),
//      }
//      let (v1, uv1, n1, v2, uv2, n2, v3, uv3, n3): (u32, u32, u32, u32, u32, u32, u32, u32, u32);
//      scan!(line.bytes() => "f {}/{}/{} {}/{}/{} {}/{}/{}", v1, uv1, n1, v2, uv2, n2, v3, uv3, n3);
//      // obj file is [1, n] indexed, uv has dummy head
//      tmp_index.push(((v1 - 1, uv1, n1 - 1), (v2 - 1, uv2, n2 - 1), (v3 - 1, uv3, n3 - 1)));
    } // else: comment, ignore
  }
  // if we don't do this, just use as (tmp_v, tmp_uv, tmp_norm, tmp_index) as result
  // it is also ok, but to determine a triangle, it needs 9 index
  // now we compress (tmp_v, tmp_uv, tmp_norm) into one index
  // at worst case this make points size grow up to 3x, but normally it is ok(for big cases like dragon.obj, 1x exactly)
  // now only need 3 index to determine a triangle
  let mut cache = HashMap::new();
  let (mut v, mut uv, mut norm, mut index) = (Vec::with_capacity(tmp_index.len() * 3), Vec::with_capacity(tmp_index.len() * 3), Vec::with_capacity(tmp_index.len() * 3), Vec::with_capacity(tmp_index.len()));
  for ((v1, uv1, n1), (v2, uv2, n2), (v3, uv3, n3)) in tmp_index {
    let p1 = (tmp_v[v1 as usize],
              tmp_uv[uv1 as usize],
              tmp_norm[n1 as usize]);
    let p2 = (tmp_v[v2 as usize], tmp_uv[uv2 as usize], tmp_norm[n2 as usize]);
    let p3 = (tmp_v[v3 as usize], tmp_uv[uv3 as usize], tmp_norm[n3 as usize]);
    index.push((
      *cache.entry(p1).or_insert_with(||
        (v.push(p1.0), uv.push(p1.1), norm.push(p1.2), v.len() as u32 - 1).3
      ),
      *cache.entry(p2).or_insert_with(||
        (v.push(p2.0), uv.push(p2.1), norm.push(p2.2), v.len() as u32 - 1).3
      ),
      *cache.entry(p3).or_insert_with(||
        (v.push(p3.0), uv.push(p3.1), norm.push(p3.2), v.len() as u32 - 1).3
      ),
    ));
  }
  Ok(Mesh::new(v, uv, norm, index))
}

pub fn bezier_curve(path: &str) -> io::Result<BezierCurve> {
  let file = File::open(path)?;
  let mut ps = Vec::new();
  for line in io::BufReader::new(file).lines() {
    let line = line?;
    let (x, y): (f64, f64);
    scan!(line.bytes() => "{} {}", x, y);
    ps.push(F64Vec3(x, y, 0.0));
  }
  Ok(BezierCurve::new(ps.into_boxed_slice()))
}

pub fn bezier_surface(path: &str) -> io::Result<BezierSurface> {
  let file = File::open(path)?;
  let mut ps = Vec::new();
  let mut lines = io::BufReader::new(file).lines();
  let (width, height): (usize, usize);
  let first_line = lines.next().unwrap()?;
  scan!(first_line.bytes() => "{} {}", width, height);
  for line in lines {
    let line = line?;
    let (x, y, z): (f32, f32, f32);
    scan!(line.bytes() => "{} {} {}", x, y, z);
    ps.push(Vec3(x, y, z));
  }
  assert_eq!(width * height, ps.len());
  Ok(BezierSurface::new(ps.into(), width))
}

pub fn texture(path: &str, flip: bool) -> io::Result<Color> {
  fn inv_gamma(x: u8) -> f32 {
    (x as f32 / 255.0).powf(2.2)
  }
  let decoder = png::Decoder::new(File::open(path)?);
  let (info, mut reader) = decoder.read_info().unwrap();
  let mut buf = vec![0; info.buffer_size()];
  reader.next_frame(&mut buf).unwrap();
  let mut color = Vec::with_capacity(buf.len() / 3);
  let mut i = 0;
  while i < buf.len() {
    color.push(Vec3(inv_gamma(buf[i]), inv_gamma(buf[i + 1]), inv_gamma(buf[i + 2])));
    i += if info.color_type == png::ColorType::RGBA { 4 } else { 3 };
  }
  if flip {
    let mut flipped = vec![Vec3::zero(); color.len()];
    let (w, h) = (info.width as usize, info.height as usize);
    for i in 0..h {
      let s = &mut flipped[i * w..(i + 1) * w];
      s.clone_from_slice(&color[(h - 1 - i) * w..(h - i) * w]);
    }
    Ok(Color::Image { data: flipped.into(), w: info.width, h: info.height })
  } else {
    Ok(Color::Image { data: color.into(), w: info.width, h: info.height })
  }
}