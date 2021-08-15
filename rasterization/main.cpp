#include <vector>
#include "pic.h"

const std::vector<Vec3> RGB_TABLE = [] {
  std::vector<Vec3> ret;
  auto range_ck = [](int x) { return x > 255 ? 255 : x < 0 ? 0 : x; };
  int r = 0, g = 0, b = 255;
  int r_f = 0, g_f = 0, b_f = 1;
  while (true) {
    ret.push_back(Vec3{r / 255.0, g / 255.0, b / 255.0});
    if (b == 255)
      g_f = 1;
    if (g == 255)
      b_f = -1;
    if (b == 0)
      r_f = +1;
    if (r == 255)
      g_f = -1;
    if (g == 0 && b == 0)
      r_f = -1;
    if (r < 127 && g == 0 && b == 0)
      break;
    r = range_ck(r + r_f);
    g = range_ck(g + g_f);
    b = range_ck(b + b_f);
  }
  return ret;
}();

Vec3 f2color(f64 x) {
  if (!(0 <= x && x <= 1))
    return Vec3{};
  size_t index = x * RGB_TABLE.size();
  index -= index == RGB_TABLE.size();
  return RGB_TABLE[index];
}

namespace draw {
  const i32 SIZE = 1000, HALF = SIZE / 2;

  void star() {
    const f64 R1 = SIZE * 0.2, R2 = SIZE * 0.4;
    std::pair<i32, i32> in[6], out[6];
    for (i32 i = 0; i < 5; ++i) {
      in[i] = {HALF + std::cos(i * (PI * 72 / 180)) * R1 + 0.5,
               HALF + std::sin(i * (PI * 72 / 180)) * R1 + 0.5};
      out[i] = {HALF + std::cos((i + 0.5) * (PI * 72 / 180)) * R2 + 0.5,
                HALF + std::sin((i + 0.5) * (PI * 72 / 180)) * R2 + 0.5};
    }
    in[5] = in[0], out[5] = out[0];
    Pic pic(SIZE, SIZE, Vec3{0.5, 0.5, 0.5});
    for (i32 i = 0; i < 5; ++i) {
      pic.line(out[i].first, out[i].second, in[i + 1].first, in[i + 1].second, Vec3{});
      pic.line(out[i].first, out[i].second, in[i].first, in[i].second, Vec3{});
    }
    pic.output_png("empty_star.png")
        .fill(HALF, HALF, [](i32 x, i32 y) { return f2color(std::sqrt((x * x + y * y) / (2.0 * SIZE * SIZE))); })
        .output_png("coarse_star.png")
        .conv(Kernel::make_gaussian(3, 3, 1.0))
        .output_png("star.png");
  }

  void func() {
    Pic(SIZE, SIZE, Vec3{0.5, 0.5, 0.5})
        .func([](f64 t) {
                f64 rho = std::sin(10 * t), sin = std::sin(t), cos = std::cos(t);
                return std::make_pair(HALF + HALF * rho * cos, HALF + HALF * rho * sin);
              }, 0, 10, 0.001,
              [](i32 x, i32 y) { return f2color(std::sqrt((x * x + y * y) / (2.0 * SIZE * SIZE))); })
        .func([](f64 t) {
                f64 rho = std::sin(10 * t), sin = std::sin(t), cos = std::cos(t);
                return std::make_pair(HALF + HALF * 0.6 * rho * cos, HALF + HALF * 0.6 * rho * sin);
              }, 0, 10, 0.001,
              [](i32 x, i32 y) { return f2color(std::sqrt((x * x + y * y) / (2.0 * SIZE * SIZE))); })
        .output_png("func.png");
  }

  void polygon() {
    const i32 N = 17;
    std::pair<i32, i32> ps[N];
    for (i32 i = 0; i < N; ++i) {
      ps[i] = {HALF + std::cos(i * (2.0 * PI / N)) * HALF + 0.5,
               HALF + std::sin(i * (2.0 * PI / N)) * HALF + 0.5};
    }
    Pic(SIZE, SIZE, Vec3{0.5, 0.5, 0.5})
        .polygon(ps, [gauss = Pic::from_png("gauss.png")](i32 x, i32 y) {
          return gauss[i32(f64(y) / SIZE * gauss.h)][i32(f64(x) / SIZE * gauss.w)];
        })
        .output_png("polygon.png");
  }
}

int main() {
  draw::star();
  draw::polygon();
  draw::func();
}