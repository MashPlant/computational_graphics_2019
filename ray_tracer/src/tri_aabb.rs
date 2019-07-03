/********************************************************/

/* AABB-triangle overlap test code                      */

/* by Tomas Akenine-Möller                              */

/* Function: int triBoxOverlap(float box_c[3],      */

/*          float box_half_sz[3],float tri_vert[3][3]); */

/* History:                                             */

/*   2001-03-05: released the code in its first version */

/*   2001-06-18: changed the order of the tests, faster */

/*                                                      */

/* Acknowledgement: Many thanks to Pierre Terdiman for  */

/* suggestions and discussions on how to optimize code. */

/* Thanks to David Hunt for finding a ">="-bug!         */

/********************************************************/

macro_rules! cross {
  ($dst: expr, $v1: expr, $v2: expr) => {
    $dst[0] = $v1[1] * $v2[2] - $v1[2] * $v2[1];
    $dst[1] = $v1[2] * $v2[0] - $v1[0] * $v2[2];
    $dst[2] = $v1[0] * $v2[1] - $v1[1] * $v2[0];
  };
}

macro_rules! dot {
  ($v1: expr, $v2: expr) => {
    $v1[0] * $v2[0] + $v1[1] * $v2[1] + $v1[2] * $v2[2]
  };
}

macro_rules! sub {
  ($dst: expr, $v1: expr, $v2: expr) => {
    $dst[0] = $v1[0] - $v2[0];
    $dst[1] = $v1[1] - $v2[1];
    $dst[2] = $v1[2] - $v2[2];
  };
}

macro_rules! min_max {
  ($x0: expr, $x1: expr, $x2: expr, $min: expr, $max: expr) => {
    $min = $x0;
    $max = $x0;
    if $x1< $min { $min = $x1; }
    if $x1> $max { $max = $x1; }
    if $x2< $min { $min = $x2; }
    if $x2> $max { $max = $x2; }
  };
}

fn plane_box_overlap(normal: [f32; 3], vert: [f32; 3], max: [f32; 3]) -> bool {
  let (mut v_min, mut v_max) = ([0.0; 3], [0.0; 3]);
  for i in 0..3 {
    let v = vert[i];
    if normal[i] > 0.0 {
      v_min[i] = -max[i] - v;
      v_max[i] = max[i] - v;
    } else {
      v_min[i] = max[i] - v;
      v_max[i] = -max[i] - v;
    }
  }
  dot!(normal, v_min) <= 0.0 && dot!(normal, v_max) >= 0.0
}

// well, anyway I don't understand how it works
pub fn tri_box_overlap(box_c: [f32; 3], box_half_sz: [f32; 3], tri_vert: [[f32; 3]; 3]) -> bool {
  let (mut v0, mut v1, mut v2, mut normal, mut e0, mut e1, mut e2) = ([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
  let (mut min, mut max, mut p0, mut p1, mut p2, mut rad, mut fex, mut fey, mut fez);
  macro_rules! test_x01 {
    ($a: expr, $b: expr, $fa: expr, $fb: expr) => {
      p0 = $a * v0[1] - $b * v0[2];
      p2 = $a * v2[1] - $b * v2[2];
      if p0 < p2 { min = p0; max = p2; } else { min = p2; max = p0; }
      rad = $fa * box_half_sz[1] + $fb * box_half_sz[2];
      if min > rad || max < -rad { return false; }
    };
  }
  macro_rules! test_x2 {
    ($a: expr, $b: expr, $fa: expr, $fb: expr) => {
      p0 = $a * v0[1] - $b * v0[2];
      p1 = $a * v1[1] - $b * v1[2];
      if p0 < p1 { min = p0; max = p1; } else { min = p1; max = p0; }
      rad = $fa * box_half_sz[1] + $fb * box_half_sz[2];
      if min > rad || max < -rad { return false; }
    };
  }
  macro_rules! test_y02 {
    ($a: expr, $b: expr, $fa: expr, $fb: expr) => {
      p0 = -$a * v0[0] + $b * v0[2];
      p2 = -$a * v2[0] + $b * v2[2];
      if p0 < p2 { min = p0; max = p2; } else { min = p2; max = p0; }
      rad = $fa * box_half_sz[0] + $fb * box_half_sz[2];
      if min > rad || max < -rad { return false; }
    };
  }
  macro_rules! test_y1 {
    ($a: expr, $b: expr, $fa: expr, $fb: expr) => {
      p0 = -$a * v0[0] + $b * v0[2];
      p1 = -$a * v1[0] + $b * v1[2];
      if p0 < p1 { min = p0; max = p1; } else { min = p1; max = p0; }
      rad = $fa * box_half_sz[0] + $fb * box_half_sz[2];
      if min > rad || max < -rad { return false; }
    };
  }
  macro_rules! test_z12 {
    ($a: expr, $b: expr, $fa: expr, $fb: expr) => {
      p1 = $a * v1[0] - $b * v1[1];
      p2 = $a * v2[0] - $b * v2[1];
      if p2 < p1 { min = p2; max = p1; } else { min = p1; max = p2; }
      rad = $fa * box_half_sz[0] + $fb * box_half_sz[1];
      if min > rad || max < -rad { return false; }
    };
  }
  macro_rules! test_z0 {
    ($a: expr, $b: expr, $fa: expr, $fb: expr) => {
      p0 = $a * v0[0] - $b * v0[1];
      p1 = $a * v1[0] - $b * v1[1];
      if p0 < p1 { min = p0; max = p1; } else { min = p1; max = p0; }
      rad = $fa * box_half_sz[0] + $fb * box_half_sz[1];
      if min > rad || max < -rad { return false; }
    };
  }
  sub!(v0, tri_vert[0], box_c);
  sub!(v1, tri_vert[1], box_c);
  sub!(v2, tri_vert[2], box_c);
  sub!(e0, v1, v0);
  sub!(e1, v2, v1);
  sub!(e2, v0, v2);
  fex = e0[0].abs();
  fey = e0[1].abs();
  fez = e0[2].abs();
  test_x01!(e0[2], e0[1], fez, fey);
  test_y02!(e0[2], e0[0], fez, fex);
  test_z12!(e0[1], e0[0], fey, fex);
  fex = e1[0].abs();
  fey = e1[1].abs();
  fez = e1[2].abs();
  test_x01!(e1[2], e1[1], fez, fey);
  test_y02!(e1[2], e1[0], fez, fex);
  test_z0!(e1[1], e1[0], fey, fex);
  fex = e2[0].abs();
  fey = e2[1].abs();
  fez = e2[2].abs();
  test_x2!(e2[2], e2[1], fez, fey);
  test_y1!(e2[2], e2[0], fez, fex);
  test_z12!(e2[1], e2[0], fey, fex);
  min_max!(v0[0], v1[0], v2[0], min, max);
  if min > box_half_sz[0] || max < -box_half_sz[0] { return false; }
  min_max!(v0[1], v1[1], v2[1], min, max);
  if min > box_half_sz[1] || max < -box_half_sz[1] { return false; }
  min_max!(v0[2], v1[2], v2[2], min, max);
  if min > box_half_sz[2] || max < -box_half_sz[2] { return false; }
  cross!(normal, e0, e1);
  if !plane_box_overlap(normal, v0, box_half_sz) { return false; }
  return true;
}