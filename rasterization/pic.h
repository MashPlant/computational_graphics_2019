#pragma once

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <algorithm>
#include <utility>
#include <vector>
#include <queue>
#include <memory>
#include <cassert>
#include "lodepng.h"

using u8 = unsigned char;
using i32 = int;
using u32 = unsigned;
using f32 = float;
using f64 = double;

constexpr f64 PI = 3.14159265358979323846264338327950288;

struct Vec3 {
  f64 x, y, z;

  bool operator==(const Vec3 &rhs) const {
    return x == rhs.x && y == rhs.y && z == rhs.z;
  }

  Vec3 &operator+=(const Vec3 &rhs) {
    x += rhs.x, y += rhs.y, z += rhs.z;
    return *this;
  }

  Vec3 operator*(f64 rhs) const {
    return {x * rhs, y * rhs, z * rhs};
  }
};

struct Kernel {
  std::unique_ptr<f64[]> a;
  i32 w, h;

  f64 &at(i32 x, i32 y) {
    return a[w * y + x];
  }

  f64 at(i32 x, i32 y) const {
    return a[w * y + x];
  }

  Kernel(i32 w, i32 h) : a(std::make_unique<f64[]>(size_t(w) * h)), w(w), h(h) {}

  static Kernel make_gaussian(i32 w, i32 h, f64 sigma) {
    Kernel kernel(w, h);
    f64 sum = 0.0;
    for (i32 i = 0; i < w; ++i) {
      for (i32 j = 0; j < h; ++j) {
        kernel.at(i, j) = std::exp(-(i * i + j * j) / (2 * sigma * sigma));
        sum += kernel.at(i, j);
      }
    }
    for (i32 i = 0; i < w * h; ++i) {
      kernel.a[i] /= sum;
    }
    return kernel;
  }
};

struct Pic {
  std::unique_ptr<Vec3[]> a;
  i32 w, h;

  void set(i32 x, i32 y, const Vec3 &col) {
    if (u32(x) < u32(w) && u32(y) < u32(h)) {
      a[w * y + x] = col;
    }
  }

  static Pic from_png(const char *path) {
    // inv gamma correction
    auto from_u8 = [](u8 x) { return std::pow(x / 255.0, 2.2); };
    u8 *data;
    u32 w, h;
    assert(lodepng_decode_file(&data, &w, &h, path, LCT_RGB, 8) == 0);
    Pic ret(w, h);
    for (i32 i = 0, cnt = 0; i < h; ++i) {
      for (i32 j = 0; j < w; ++j) {
        ret[i][j] = {from_u8(data[cnt]), from_u8(data[cnt + 1]), from_u8(data[cnt + 2])};
        cnt += 3;
      }
    }
    free(data);
    return ret;
  }

  Pic &output_png(const char *path) {
    // gamma correction
    auto to_u8 = [](f64 x) { return u8(std::pow(x < 0.0 ? 0.0 : x > 1.0 ? 1.0 : x, 1.0 / 2.2) * 255.0 + 0.5); };
    FILE *fp = fopen(path, "w");
    std::unique_ptr<u8[]> png = std::make_unique<u8[]>(size_t(w) * h * 3);
    for (i32 i = 0, cnt = 0; i < h; ++i) {
      for (i32 j = 0; j < w; ++j) {
        png[cnt] = to_u8(a[i * w + j].x);
        png[cnt + 1] = to_u8(a[i * w + j].y);
        png[cnt + 2] = to_u8(a[i * w + j].z);
        cnt += 3;
      }
    }
    assert(lodepng_encode_file(path, png.get(), w, h, LCT_RGB, 8) == 0);
    fclose(fp);
    return *this;
  }

  Vec3 *operator[](i32 y) {
    return &a[w * y];
  }

  const Vec3 *operator[](i32 y) const {
    return &a[w * y];
  }

  Pic &line(i32 x1, i32 y1, i32 x2, i32 y2, const Vec3 &col) {
    return line(x1, y1, x2, y2, [&col](i32, i32) { return col; });
  }

  template<typename F /* Fn(i32, i32) -> Vec3 */ >
  Pic &line(i32 x1, i32 y1, i32 x2, i32 y2, F f) {
    if (std::abs(y2 - y1) < std::abs(x2 - x1)) {
      if (x1 > x2) {
        std::swap(x1, x2);
        std::swap(y1, y2);
      }
      f64 k = 1.0 * (y2 - y1) / (x2 - x1), y = y1;
      for (; x1 <= std::min(x2, w - 1); ++x1, y += k) {
        set(x1, i32(y + 0.5), f(x1, i32(y + 0.5)));
      }
    } else {
      if (y1 > y2) {
        std::swap(x1, x2);
        std::swap(y1, y2);
      }
      f64 k = 1.0 * (x2 - x1) / (y2 - y1), x = x1;
      for (; y1 <= std::min(y2, h - 1); ++y1, x += k) {
        set(i32(x + 0.5), y1, f(i32(x + 0.5), y1));
      }
    }
    return *this;
  }

  Pic &fill(i32 x, i32 y, const Vec3 &col) {
    return fill(x, y, [&col](i32, i32) { return col; });
  }

  template<typename F /* Fn(i32, i32) -> Vec3 */ >
  Pic &fill(i32 init_x, i32 init_y, F f) {
    constexpr i32 dx[] = {1, 0, -1, 0};
    constexpr i32 dy[] = {0, 1, 0, -1};
    std::queue<std::pair<i32, i32>> q;
    if (init_x < w && init_y < h) {
      Vec3 center = a[init_y * w + init_x];
      q.push({init_x, init_y});
      while (!q.empty()) {
        auto[x, y] = q.front();
        q.pop();
        if (a[w * y + x] == center) {
          for (i32 i = 0; i < 4; ++i) {
            i32 nx = x + dx[i], ny = y + dy[i];
            if (u32(nx) < u32(w) && u32(ny) < u32(h)) {
              q.push({nx, ny});
            }
          }
        }
        a[y * w + x] = f(x, y);
      }
    }
    return *this;
  }

  template<typename Pr /* Pair<i32, i32> */, i32 N>
  Pic &polygon(const Pr (&ps)[N], const Vec3 &col) {
    return polygon(ps, [&col](i32, i32) { return col; });
  }

  template<typename Pr /* Pair<i32, i32> */, i32 N, typename F /* Fn(i32, i32) -> Vec3 */ >
  Pic &polygon(const Pr (&ps)[N], F f) {
    i32 min_y = 0, max_y = h - 1;
    for (auto[_, y] : ps) {
      min_y = std::min(min_y, y);
      max_y = std::max(max_y, y);
    }
    for (i32 y = min_y; y <= max_y; ++y) {
      i32 cnt = 0, xs[N];
      for (i32 i = 0; i < N; ++i) {
        auto[pre, cur] = i ? std::pair(i - 1, i) : std::pair(N - 1, 0);
        auto[x1, y1] = ps[pre];
        auto[x2, y2] = ps[cur];
        if (y1 > y2) {
          std::swap(x1, x2);
          std::swap(y1, y2);
        } else if (y1 == y2) {
          continue;
        }
        if (y >= y1 && y < y2) {
          xs[cnt++] = (y - y1) * (x2 - x1) / (y2 - y1) + x1;
        } else if ((y == max_y) && (y > y1) && (y <= y2)) {
          xs[cnt++] = (y - y1) * (x2 - x1) / (y2 - y1) + x1;
        }
      }
      std::sort(xs, xs + cnt);
      for (i32 i = 0; i + 1 < cnt; i += 2) {
        for (i32 j = std::max(0, xs[i]), end = std::min(w - 1, xs[i + 1]); j <= end; ++j) {
          a[y * w + j] = f(j, y);
        }
      }
    }
    return *this;
  }

  Pic conv(const Kernel &kernel) const {
    Pic pic(w - kernel.w + 1, h - kernel.h + 1);
#pragma omp parallel for
    for (i32 i = 0; i < pic.h; ++i) {
      for (i32 j = 0; j < pic.w; ++j) {
        Vec3 sum{};
        for (i32 p = i; p < i + kernel.h; ++p) {
          for (i32 q = j; q < j + kernel.w; ++q) {
            sum += a[p * w + q] * kernel.at(q - j, p - i);
          }
        }
        pic.a[i * pic.w + j] = sum;
      }
    }
    return pic;
  }

  template<typename F /* Fn(f64) -> (i32, i32) */ >
  Pic &func(F f, f64 s, f64 e, f64 step, const Vec3 &col) {
    auto[x0, y0] = f(s);
    for (; (s += step) <= e;) {
      auto[x, y] = f(s);
      line(x0, y0, x, y, col);
      x0 = x, y0 = y;
    }
    return *this;
  }

  template<typename F /* Fn(f64) -> (i32, i32) */, typename G /* Fn(i32, i32) -> Vec3 */  >
  Pic &func(F f, f64 s, f64 e, f64 step, G g) {
    auto[x0, y0] = f(s);
    for (; (s += step) <= e;) {
      auto[x, y] = f(s);
      line(x0, y0, x, y, g);
      x0 = x, y0 = y;
    }
    return *this;
  }

  Pic(i32 w, i32 h, const Vec3 &bg = Vec3{}) : a(std::make_unique<Vec3[]>(size_t(w) * h)), w(w), h(h) {
    std::fill(a.get(), a.get() + w * h, bg);
  }
};

