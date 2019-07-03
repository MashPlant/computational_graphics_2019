use super::util::*;
use super::vec::*;

pub struct PPM(String);

fn compress_256(x: f32) -> u8 {
  (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

impl PPM {
  pub fn new(data: &Vec<Vec3>, w: u32, h: u32) -> PPM {
    let mut ppm = format!("P3\n{} {}\n{}\n", w, h, 255);
    for v in data {
      ppm += &format!("{} {} {} ", compress_256(v.0), compress_256(v.1), compress_256(v.2));
    }
    PPM(ppm)
  }

  pub fn data(&self) -> &[u8] {
    self.0.as_bytes()
  }
}

pub struct PNG {
  data: Vec<u8>,
  w: u32,
  h: u32,
}

impl PNG {
  pub fn new(data: &Vec<Vec3>, w: u32, h: u32) -> PNG {
    let mut rgb = Vec::with_capacity(data.len() * 3);
    for v in data {
      rgb.push(compress_256(v.0));
      rgb.push(compress_256(v.1));
      rgb.push(compress_256(v.2));
    }
    PNG { data: rgb, w, h, }
  }

  pub fn write(&self, path: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::BufWriter;
    use png::HasParameters;

    let file = File::create(path)?;
    let mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(&mut w, self.w, self.h);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&self.data).unwrap();
    Ok(())
  }
}