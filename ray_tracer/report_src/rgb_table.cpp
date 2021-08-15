#include <cmath>
#include <cstdio>
#include <vector>

using f32 = float;
using u32 = unsigned;

struct Vec3 {
  f32 x, y, z;
};

std::vector<Vec3> RGB_TABLE = [] {
  std::vector<Vec3> ret;
  auto range_ck = [](int x) { return x > 255 ? 255 : x < 0 ? 0 : x; };
  int r = 0, g = 0, b = 255;
  int r_f = 0, g_f = 0, b_f = 1;
  while (true) {
    ret.push_back(Vec3{r / 255.0f, g / 255.0f, b / 255.0f});
    if (b == 255) g_f = 1;
    if (g == 255) b_f = -1;
    if (b == 0) r_f = +1;
    if (r == 255) g_f = -1;
    if (g == 0 && b == 0) r_f = -1;
    if (r < 127 && g == 0 && b == 0) break;
    r = range_ck(r + r_f);
    g = range_ck(g + g_f);
    b = range_ck(b + b_f);
  }
  return ret;
}();

int main() {
  const u32 RGB_SAMPLE = 32;
  f32 r = 0, g = 0, b = 0;
  for (u32 i = 0; i < RGB_SAMPLE; ++i) {
    u32 idx = (1150 / RGB_SAMPLE) * i;
    Vec3 &v = RGB_TABLE[idx];
    v.x = powf(v.x, 2.2f);
    v.y = powf(v.y, 2.2f);
    v.z = powf(v.z, 2.2f);
    r += v.x;
    g += v.y;
    b += v.z;
  }
  // for (auto &v : RGB_TABLE) {
  //   // f32 max = fmaxf(v.x, fmaxf(v.y, v.z));
  //   v.x = powf(v.x, 2.2f);
  //   v.y = powf(v.y, 2.2f);
  //   v.z = powf(v.z, 2.2f);
  //   r += v.x;
  //   g += v.y;
  //   b += v.z;
  // }
  fprintf(stderr, "%f %f %f\n", r, g, b);
  f32 fac_g = r / g, fac_b = r / b;
  printf("CONSTANT const Vec3 RGB_TABLE[] = {");
  for (u32 i = 0; i < RGB_SAMPLE; ++i) {
    u32 idx = (1150 / RGB_SAMPLE) * i;
    Vec3 v = RGB_TABLE[idx];
    printf("Vec3{%f, %f, %f}, ", v.x, v.y * fac_g, v.z * fac_b);
  }
  printf("};");
}