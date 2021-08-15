pub fn clamp(x: f32) -> f32 {
  if x < 0.0 { 0.0 } else { if x > 1.0 { 1.0 } else { x } }
}

pub const EPS: f32 = 1e-3;

pub struct XorShiftRng {
  seed: u32,
}

impl XorShiftRng {
  pub fn new(seed: u32) -> XorShiftRng {
    XorShiftRng { seed: if seed == 0 { 1 } else { seed } }
  }

  pub fn gen(&mut self) -> f32 {
    const INV_U32_MAX: f32 = 1.0 / std::u32::MAX as f32;
    let mut x = self.seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.seed = x;
    return x as f32 * INV_U32_MAX;
  }
}