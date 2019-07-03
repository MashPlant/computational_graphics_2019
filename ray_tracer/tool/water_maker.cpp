#include "tracer_util.hpp"
#include <random>

const u32 SAMPLE_X = 200;
const u32 SAMPLE_Y = 200;
const f32 D_X = 1.0f / SAMPLE_X;
const f32 D_Y = 1.0f / SAMPLE_Y;
const u32 WAVE_N = 100;

struct SinWave {
  Vec2 c;
  f32 A, v, phi;

  f32 value(Vec2 xy) const { return A * sin(phi + (xy - c).len() / v); }
} ws[WAVE_N];

f32 ss[SAMPLE_X + 1][SAMPLE_Y + 1];
char buf[1 << 20];

int main(int argc, char **argv) {
  if (argc != 3) {
    puts("usage: ./a.out mx my");
    exit(-1);
  }
  f32 mx = atof(argv[1]), my = atof(argv[2]);
  std::mt19937 mt{19260817};
  std::uniform_real_distribution<f32> uni;
  using UniParm = decltype(uni)::param_type;
  for (u32 i = 0; i < WAVE_N; ++i) {
    ws[i].c = {uni(mt, UniParm{0, mx}), uni(mt, UniParm{0, my})};
    ws[i].A = 0.001f * mx; // uni(mt, UniParm{0.001f * mx, 0.0012f * mx});
    ws[i].v = 0.3f;
    // ws[i].v = 0.3f; // adjust it !
    ws[i].phi = uni(mt, UniParm{0, 2.0f * PI});
  }
#pragma omp paraller for
  for (u32 i = 0; i <= SAMPLE_X; ++i) {
    for (u32 j = 0; j <= SAMPLE_Y; ++j) {
      Vec2 xy{i * D_X * mx, j * D_Y * my};
      for (u32 k = 0; k < WAVE_N; ++k) {
        ss[i][j] += ws[k].value(xy);
      }
    }
  }
  setvbuf(stdout, buf, _IOFBF, sizeof buf);
  // to fit the coordinate in my ray tracer
  // the wave is on xz plane(only affect v & vn)
  for (u32 i = 0; i < SAMPLE_X; ++i) {
    for (u32 j = 0; j < SAMPLE_Y; ++j) {
      printf("v %.6f %.6f %.6f\n", i * D_X * mx, ss[i][j], j * D_Y * my);
    }
  }
  for (u32 i = 0; i < SAMPLE_X; ++i) {
    for (u32 j = 0; j < SAMPLE_Y; ++j) {
      printf("vt %.6f %.6f\n", i * D_X, j * D_Y);
    }
  }
  for (u32 i = 0; i < SAMPLE_X; ++i) {
    for (u32 j = 0; j < SAMPLE_Y; ++j) {
      Vec3 dx{D_X * mx, ss[i + 1][j] - ss[i][j], 0.0f};
      Vec3 dy{0.0f, ss[i][j + 1] - ss[i][j], D_Y * my};
      Vec3 n = dy.cross(dx).norm();
      printf("vn %.6f %.6f %.6f\n", n.x, n.y, n.z);
    }
  }
  for (u32 i = 0; i < SAMPLE_X - 1; ++i) {
    for (u32 j = 0; j < SAMPLE_Y - 1; ++j) {
      u32 a = i * SAMPLE_Y + j + 1, b = a + 1, c = a + SAMPLE_Y, d = c + 1;
      printf("f %d/%d/%d %d/%d/%d %d/%d/%d\n", a, a, a, b, b, b, d, d, d);
      printf("f %d/%d/%d %d/%d/%d %d/%d/%d\n", a, a, a, c, c, c, d, d, d);
    }
  }
}