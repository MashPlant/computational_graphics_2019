#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <algorithm>
#include <cassert>
#include <cstdio>
#include <memory>
#include <vector>
#include "cmdline/cmdline.h"
#include "lodepng/lodepng.h"

using i8 = char;
using u8 = unsigned char;
using i32 = int;
using u32 = unsigned;
using u64 = unsigned long long;
using f32 = float;
using f64 = double;

struct Vec3 {
  u32 x, y, z;

  Vec3 operator+(const Vec3 &rhs) const {
    return Vec3{x + rhs.x, y + rhs.y, z + rhs.z};
  }

  Vec3 operator-(const Vec3 &rhs) const {
    return Vec3{x - rhs.x, y - rhs.y, z - rhs.z};
  }

  Vec3 operator*(u32 rhs) const {
    return Vec3{x * rhs, y * rhs, z * rhs};
  }

  i32 abs() const {
    return u32(std::abs((i32) x) + std::abs((i32) y) + std::abs((i32) z));
  }
};

struct Color {
  u8 r, g, b;

  Vec3 vec3() const {
    return Vec3{(u32) r, (u32) g, (u32) b};
  }

  bool operator==(const Color &rhs) const {
    return r == rhs.r && g == rhs.g && b == rhs.b;
  }

  bool operator!=(const Color &rhs) const {
    return !(*this == rhs);
  }
};

struct Rect {
  i32 x1, y1, x2, y2;
};

namespace seam {
  std::vector<u32> idx;
  std::vector<Color> col;
  u32 w, h;

  void init(u32 w, u32 h) {
    seam::w = w;
    seam::h = h;
    idx.resize(w * h);
    col.resize(w * h);
    for (u32 i = 0; i < w * h; ++i) {
      idx[i] = i;
    }
  }

  void rotate_col() {
    std::vector<Color> tmp(col.size());
    for (u32 i = 0; i < w; ++i) {
      for (u32 j = 0; j < h; ++j) {
        tmp[i * h + j] = col[j * w + i];
      }
    }
    col = tmp;
    std::swap(w, h);
  }

  void rotate_idx() {
    std::vector<u32> tmp(idx.size());
    for (u32 i = 0; i < w; ++i) {
      for (u32 j = 0; j < h; ++j) {
        tmp[i * h + j] = idx[j * w + i];
      }
    }
    idx = tmp;
    std::swap(w, h);
  }
}

struct Pic {
  struct FreeDel {
    void operator()(void *p) const {
      free(p);
    }
  };

  template<typename T>
  using Arr = std::unique_ptr<T[], FreeDel>;

  Arr<u8> a;
  u32 w, h;

  Pic(u32 w, u32 h) : a((u8 *) malloc(sizeof(Color) * w * h)), w(w), h(h) {}

  explicit Pic(const char *file) {
    i32 fd = open(file, O_RDONLY);
    assert(fd != -1 && "fail to open file");
    struct stat st;
    fstat(fd, &st);
    u8 *img = (u8 *) mmap(0, st.st_size, PROT_READ, MAP_PRIVATE, fd, 0), *data;
    assert(lodepng_decode24(&data, &w, &h, img, st.st_size) == 0 && "fail to decode png");
    close(fd);
    munmap(data, st.st_size);
    a.reset(data);
  }

  Color *operator[](u32 index) {
    return (Color *) (a.get() + w * index * 3);
  }

  const Color *operator[](u32 index) const {
    return (const Color *) (a.get() + w * index * 3);
  }

  void output(const char *file) const {
    assert(lodepng_encode_file(file, a.get(), w, h, LCT_RGB, 8) == 0);
  }

  Pic col_carve(f32 fac) const {
    assert(fac < 1.0 && fac > 0.0);
    i32 w = this->w, h = this->h;
    i32 pad_w = w + 2, pad_h = h + 2;
    i32 wh = w * h, pad_wh = pad_w * pad_h;
    i32 nw = (i32) (fac * w);
    Arr<Color> _new_pic((Color *) calloc((u32) pad_wh, sizeof(Color)));
    // not compress space for dp, use it to trace move
    Arr<u32> _e_map((u32 *) malloc(wh * sizeof(u32))), _dp((u32 *) calloc((u32) pad_wh, sizeof(u32)));
    Arr<i32> move((i32 *) malloc(h * sizeof(i32)));
    Color *new_pic = _new_pic.get() + pad_w + 1;
    u32 *e_map = _e_map.get(), *dp = _dp.get() + pad_w + 1;
    for (i32 i = 0; i < h; ++i) {
      memcpy(new_pic + i * pad_w, a.get() + 3 * i * w, w * sizeof(Color));
      dp[i * pad_w - 1] = 0x3f3f3f3f;
    }

#if LAPLACE
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[i * pad_w + j].vec3() * 4 -
              new_pic[i * pad_w + j - 1].vec3() -
              new_pic[i * pad_w + j + 1].vec3() -
              new_pic[(i - 1) * pad_w + j].vec3() -
              new_pic[(i + 1) * pad_w + j].vec3()).abs();
    };
#elif ROBERTS
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[i * pad_w + j].vec3() - new_pic[(i + 1) * pad_w + j + 1].vec3()).abs() +
             (new_pic[(i + 1) * pad_w + j].vec3() - new_pic[i * pad_w + (j + 1)].vec3()).abs();
    };
#else
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[(i - 1) * pad_w + j - 1].vec3() + new_pic[(i - 1) * pad_w + j].vec3() * 2 +
              new_pic[(i - 1) * pad_w + j + 1].vec3()
              - new_pic[(i + 1) * pad_w + j - 1].vec3() - new_pic[(i + 1) * pad_w + j].vec3() * 2 -
              new_pic[(i + 1) * pad_w + j + 1].vec3()).abs()
             + (new_pic[(i - 1) * pad_w + j - 1].vec3() + new_pic[i * pad_w + j - 1].vec3() * 2 +
                new_pic[(i + 1) * pad_w + j - 1].vec3()
                - new_pic[(i - 1) * pad_w + j + 1].vec3() - new_pic[i * pad_w + j + 1].vec3() * 2 -
                new_pic[(i + 1) * pad_w + j + 1].vec3()).abs();
    };
#endif

    // pre calculate e_map & dp, many parts of them will not change during calculation
#pragma omp parallel for
    for (i32 i = 0; i < h; ++i) {
      for (i32 j = 0; j < w; ++j) {
        e_map[i * w + j] = kernel(i, j);
      }
    }

    for (i32 wx = w; wx > nw; --wx) {
      for (i32 i = 0; i < h; ++i) {
        dp[i * pad_w + wx] = 0x3f3f3f3f;
      }
      const i32 TH = 4;
#pragma omp parallel num_threads(TH)
      for (i32 i = 0; i < h; i += (wx / TH + 1) / 2) {
#pragma omp for
        for (i32 tid = 0; tid < TH; ++tid) {
          i32 beg = wx / TH * tid, end = tid == TH - 1 ? wx : wx / TH * (tid + 1);
          for (i32 ext = 0; ext < std::min(h - i, (wx / TH + 1) / 2); ++ext) {
            u32 *p1 = dp + (i - 1 + ext) * pad_w, *p2 = dp + (i + ext) * pad_w, *p3 = e_map + (i + ext) * w;
            for (i32 j = beg + ext; j < end - ext; ++j) {
              p2[j] = std::min(p1[j - 1], std::min(p1[j], p1[j + 1])) + p3[j];
            }
          }
        }
#pragma omp for
        for (i32 tid = 0; tid < TH; ++tid) {
          if (tid == 0) {
            for (i32 ext = 1; ext < std::min(h - i, (wx / TH + 1) / 2); ++ext) {
              u32 *p1 = dp + (i - 1 + ext) * pad_w, *p2 = dp + (i + ext) * pad_w, *p3 = e_map + (i + ext) * w;
              for (i32 j = 0; j < ext; ++j) {
                p2[j] = std::min(p1[j - 1], std::min(p1[j], p1[j + 1])) + p3[j];
              }
              for (i32 j = wx - ext; j < wx; ++j) {
                p2[j] = std::min(p1[j - 1], std::min(p1[j], p1[j + 1])) + p3[j];
              }
            }
          } else {
            i32 beg = wx / TH * tid;
            for (i32 ext = 1; ext < std::min(h - i, (wx / TH + 1) / 2); ++ext) {
              u32 *p1 = dp + (i - 1 + ext) * pad_w, *p2 = dp + (i + ext) * pad_w, *p3 = e_map + (i + ext) * w;
              for (i32 j = beg - ext; j < beg + ext; ++j) {
                p2[j] = std::min(p1[j - 1], std::min(p1[j], p1[j + 1])) + p3[j];
              }
            }
          }
        }
      }
      {
        u32 min = 0x3f3f3f3f;
        i32 arg_min = 0;
        for (i32 i = 0; i < wx; ++i) {
          if (dp[(h - 1) * pad_w + i] < min) {
            min = dp[(h - 1) * pad_w + i];
            arg_min = i;
          }
        }
        for (i32 i = h - 1; i >= 0; --i) {
          move[i] = arg_min;
          u32 prev = dp[i * pad_w + arg_min] - e_map[i * w + arg_min];
          arg_min = prev == dp[(i - 1) * pad_w + arg_min - 1] ? arg_min - 1 :
                    prev == dp[(i - 1) * pad_w + arg_min] ? arg_min : arg_min + 1;
        }
      }

#pragma omp parallel for num_threads(4)
      for (i32 i = 0; i < h; ++i) {
        i32 arg_min = move[i];
        memmove(new_pic + i * pad_w + arg_min, new_pic + i * pad_w + arg_min + 1, (wx - 1 - arg_min) * sizeof(Color));
        new_pic[i * pad_w + wx - 1] = Color{};
        memmove(e_map + i * w + arg_min, e_map + i * w + arg_min + 1, (wx - 1 - arg_min) * sizeof(u32));
#if NEED_SEAM
        seam::col[seam::idx[i * seam::w + arg_min]] = Color{0, 255, 0};
        memmove(&seam::idx[i * seam::w + arg_min], &seam::idx[i * seam::w + arg_min + 1], (seam::w - 1 - arg_min) * sizeof(u32));
#endif
      }

#pragma omp parallel for num_threads(4)
      for (i32 i = 0; i < h; ++i) {
        i32 arg_min = move[i];
        e_map[i * w + arg_min - 1] = kernel(i, arg_min - 1);
        e_map[i * w + arg_min] = kernel(i, arg_min);
      }
    }

    Pic ret((u32) nw, (u32) h);
    for (i32 i = 0; i < h; ++i) {
      for (i32 j = 0; j < nw; ++j) {
        ret[i][j] = new_pic[i * pad_w + j];
      }
    }
    return ret;
  }

  Pic col_extend(f32 fac) const {
    assert(fac > 1.0);
    const f32 STEP = 0.15;
    i32 w = this->w, h = this->h;
    i32 nw = (i32) (fac * w), pad_nw = nw + 2;
    i32 nwh = nw * h, pad_nwh = pad_nw * (h + 2);

    Arr<Color> _new_pic((Color *) calloc((u32) pad_nwh, sizeof(Color)));
    Arr<u32> _e_map((u32 *) malloc(nwh * sizeof(u32))), _dp((u32 *) calloc((u32) pad_nwh, sizeof(u32)));
    Arr<i32> move((i32 *) malloc(u32(STEP * w) * h * sizeof(i32)));

    Color *new_pic = _new_pic.get() + pad_nw + 1;
    u32 *e_map = _e_map.get(), *dp = _dp.get() + pad_nw + 1;
    for (i32 i = 0; i < h; ++i) {
      memcpy(new_pic + i * pad_nw, a.get() + 3 * i * w, w * sizeof(Color));
      dp[i * pad_nw - 1] = 0x3f3f3f3f;
    }
#if LAPLACE
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[i * pad_nw + j].vec3() * 4 -
              new_pic[i * pad_nw + j - 1].vec3() -
              new_pic[i * pad_nw + j + 1].vec3() -
              new_pic[(i - 1) * pad_nw + j].vec3() -
              new_pic[(i + 1) * pad_nw + j].vec3()).abs();
    };
#elif ROBERTS
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[i * pad_nw + j].vec3() - new_pic[(i + 1) * pad_nw + j + 1].vec3()).abs() +
             (new_pic[(i + 1) * pad_nw + j].vec3() - new_pic[i * pad_nw + (j + 1)].vec3()).abs();
    };
#else
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[(i - 1) * pad_nw + j - 1].vec3() + new_pic[(i - 1) * pad_nw + j].vec3() * 2 +
              new_pic[(i - 1) * pad_nw + j + 1].vec3()
              - new_pic[(i + 1) * pad_nw + j - 1].vec3() - new_pic[(i + 1) * pad_nw + j].vec3() * 2 -
              new_pic[(i + 1) * pad_nw + j + 1].vec3()).abs() +
             (new_pic[(i - 1) * pad_nw + j - 1].vec3() + new_pic[i * pad_nw + j - 1].vec3() * 2 +
              new_pic[(i + 1) * pad_nw + j - 1].vec3()
              - new_pic[(i - 1) * pad_nw + j + 1].vec3() - new_pic[i * pad_nw + j + 1].vec3() * 2 -
              new_pic[(i + 1) * pad_nw + j + 1].vec3()).abs();
    };
#endif

    for (i32 wx = w; wx < nw; wx += i32(STEP * w)) {
#pragma omp parallel for
      for (i32 i = 0; i < h; ++i) {
        for (i32 j = 0; j < wx; ++j) {
          e_map[i * nw + j] = kernel(i, j);
        }
      }
      i32 step = std::min(wx + i32(STEP * w), nw) - wx;
      for (i32 s = 0; s < step; ++s) {
        for (i32 i = 0; i < h; ++i) {
          dp[i * pad_nw + wx] = 0x3f3f3f3f;
        }
        for (i32 i = 0; i < h; ++i) {
          u32 *p1 = dp + (i - 1) * pad_nw, *p2 = dp + i * pad_nw, *p3 = e_map + i * nw;
          for (i32 j = 0; j < wx; ++j) {
            p2[j] = std::min(p1[j - 1], std::min(p1[j], p1[j + 1])) + p3[j];
          }
        }

        u32 min = 0x3f3f3f3f;
        i32 arg_min = 0;
        for (i32 i = 0; i < wx; ++i) {
          if (dp[(h - 1) * pad_nw + i] < min) {
            min = dp[(h - 1) * pad_nw + i];
            arg_min = i;
          }
        }

        for (i32 i = h - 1; i >= 0; --i) {
          move[i * step + s] = arg_min;
          u32 prev = dp[i * pad_nw + arg_min] - e_map[i * nw + arg_min];
          e_map[i * nw + arg_min] = 0x3f3f3f3f;
          arg_min = prev == dp[(i - 1) * pad_nw + arg_min - 1] ? arg_min - 1 :
                    prev == dp[(i - 1) * pad_nw + arg_min] ? arg_min : arg_min + 1;
        }
      }

#pragma omp parallel for
      for (i32 i = 0; i < h; ++i) {
        std::sort(move.get() + i * step, move.get() + (i + 1) * step);
        i32 nxt = wx;
        for (i32 j = step - 1; j >= 0; --j) {
          i32 cur = move[i * step + j];
          Color l = new_pic[i * pad_nw + cur - 1], r = new_pic[i * pad_nw + cur];
          Color ave{u8((l.r + r.r) / 2), u8((l.g + r.g) / 2), u8((l.b + r.b) / 2)};
          memmove(new_pic + i * pad_nw + cur + j + 1, new_pic + i * pad_nw + cur, (nxt - cur) * sizeof(Color));
          new_pic[i * pad_nw + cur + j] = ave;
#if NEED_SEAM
          memmove(&seam::col[i * seam::w + cur + j + 1], &seam::col[i * seam::w + cur], (nxt - cur) * sizeof(Color));
          seam::col[i * seam::w + cur + j] = Color{0, 0, 255};
#endif
          nxt = cur;
        }
      }
    }
    Pic ret((u32) nw, (u32) h);
    for (i32 i = 0; i < h; ++i) {
      for (i32 j = 0; j < nw; ++j) {
        ret[i][j] = new_pic[i * pad_nw + j];
      }
    }
    return ret;
  }

  Pic rotate() const {
    Pic tmp(h, w);
    for (u32 i = 0; i < w /* tmp.h */; ++i) {
      for (u32 j = 0; j < h /* tmp.w */; ++j) {
        tmp[i][j] = (*this)[j][i];
      }
    }
    return tmp;
  }

  static void output_e_map(const u32 *e_map, u32 w, u32 h, const char *file) {
    Arr<Color> tmp((Color *) calloc(w * h, sizeof(Color)));
    for (u32 i = 0; i < h; ++i) {
      for (u32 j = 0; j < w; ++j) {
        tmp[i * w + j].g = i32(e_map[i * w + j]) > 0 ? e_map[i * w + j] / 10 : 0;
      }
    }
    lodepng_encode_file(file, (u8 *) tmp.get(), w, h, LCT_RGB, 8);
  }

  Pic rect(f32 fac, Rect r, bool is_remove) const {
    assert(fac < 1.0 && fac > 0.0);
    const u32 SPECIAL_E = is_remove ? -20000 : 20000;

    if (r.x2 < r.x1) {
      std::swap(r.x2, r.x1);
    }
    if (r.y2 < r.y1) {
      std::swap(r.y2, r.y1);
    }

    i32 w = this->w, h = this->h;
    i32 pad_w = w + 2, pad_h = h + 2;
    i32 wh = w * h, pad_wh = pad_w * pad_h;
    i32 nw = (i32) (fac * w);

    Arr<Color> _new_pic((Color *) calloc((u32) pad_wh, sizeof(Color)));
    // not compress space for dp, use it to trace move
    Arr<u32> _e_map((u32 *) malloc(wh * sizeof(u32)));
    Arr<i32> move((i32 *) malloc(h * sizeof(i32))), _dp((i32 *) calloc((u32) pad_wh, sizeof(u32)));
    Arr<bool> remove((bool *) calloc(wh, sizeof(bool)));
    for (i32 i = r.y1; i < r.y2; ++i) {
      memset(remove.get() + i * w + r.x1, true, (r.x2 - r.x1) * sizeof(bool));
    }

    Color *new_pic = _new_pic.get() + pad_w + 1;
    u32 *e_map = _e_map.get();
    i32 *dp = _dp.get() + pad_w + 1;
    for (i32 i = 0; i < h; ++i) {
      memcpy(new_pic + i * pad_w, a.get() + 3 * i * w, w * sizeof(Color));
      dp[i * pad_w - 1] = 0x3f3f3f3f;
    }

#if LAPLACE
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[i * pad_w + j].vec3() * 4 -
              new_pic[i * pad_w + j - 1].vec3() -
              new_pic[i * pad_w + j + 1].vec3() -
              new_pic[(i - 1) * pad_w + j].vec3() -
              new_pic[(i + 1) * pad_w + j].vec3()).abs();
    };
#elif ROBERTS
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[i * pad_w + j].vec3() - new_pic[(i + 1) * pad_w + j + 1].vec3()).abs() +
             (new_pic[(i + 1) * pad_w + j].vec3() - new_pic[i * pad_w + (j + 1)].vec3()).abs();
    };
#else
    auto kernel = [=](i32 i, i32 j) {
      return (new_pic[(i - 1) * pad_w + j - 1].vec3() + new_pic[(i - 1) * pad_w + j].vec3() * 2 +
              new_pic[(i - 1) * pad_w + j + 1].vec3()
              - new_pic[(i + 1) * pad_w + j - 1].vec3() - new_pic[(i + 1) * pad_w + j].vec3() * 2 -
              new_pic[(i + 1) * pad_w + j + 1].vec3()).abs()
             + (new_pic[(i - 1) * pad_w + j - 1].vec3() + new_pic[i * pad_w + j - 1].vec3() * 2 +
                new_pic[(i + 1) * pad_w + j - 1].vec3()
                - new_pic[(i - 1) * pad_w + j + 1].vec3() - new_pic[i * pad_w + j + 1].vec3() * 2 -
                new_pic[(i + 1) * pad_w + j + 1].vec3()).abs();
    };
#endif

    // pre calculate e_map & dp, many parts of them will not change during calculation
#pragma omp parallel for
    for (i32 i = 0; i < h; ++i) {
      for (i32 j = 0; j < w; ++j) {
        e_map[i * w + j] = remove[i * w + j] ? SPECIAL_E : kernel(i, j);
      }
    }

    for (i32 wx = w; wx > nw; --wx) {
      for (i32 i = 0; i < h; ++i) {
        dp[i * pad_w + wx] = 0x3f3f3f3f;
      }
      for (i32 i = 0; i < h; ++i) {
        i32 *p1 = dp + (i - 1) * pad_w, *p2 = dp + i * pad_w;
        u32 *p3 = e_map + i * w;
        for (i32 j = 0; j < wx; ++j) {
          p2[j] = std::min(p1[j - 1], std::min(p1[j], p1[j + 1])) + p3[j];
        }
      }
      {
        i32 min = 0x3f3f3f3f;
        i32 arg_min = 0;
        for (i32 i = 0; i < wx; ++i) {
          if (dp[(h - 1) * pad_w + i] < min) {
            min = dp[(h - 1) * pad_w + i];
            arg_min = i;
          }
        }
        for (i32 i = h - 1; i >= 0; --i) {
          move[i] = arg_min;
          i32 prev = dp[i * pad_w + arg_min] - e_map[i * w + arg_min];
          arg_min = prev == dp[(i - 1) * pad_w + arg_min - 1] ? arg_min - 1 :
                    prev == dp[(i - 1) * pad_w + arg_min] ? arg_min : arg_min + 1;
        }
      }
#pragma omp parallel for
      for (i32 i = 0; i < h; ++i) {
        i32 arg_min = move[i];
        memmove(new_pic + i * pad_w + arg_min, new_pic + i * pad_w + arg_min + 1, (wx - 1 - arg_min) * sizeof(Color));
        new_pic[i * pad_w + wx - 1] = Color{};
        memmove(remove.get() + i * w + arg_min, remove.get() + i * w + arg_min + 1,
                (wx - 1 - arg_min) * sizeof(bool));
        memmove(e_map + i * w + arg_min, e_map + i * w + arg_min + 1, (wx - 1 - arg_min) * sizeof(u32));
#if NEED_SEAM
        seam::col[seam::idx[i * seam::w + arg_min]] = Color{0, 255, 0};
        memmove(&seam::idx[i * seam::w + arg_min], &seam::idx[i * seam::w + arg_min + 1], (seam::w - 1 - arg_min) * sizeof(u32));
#endif
      }

#pragma omp parallel for
      for (i32 i = 0; i < h; ++i) {
        i32 arg_min = move[i];
        e_map[i * w + arg_min - 1] = remove[i * w + arg_min - 1] ? SPECIAL_E : kernel(i, arg_min - 1);
        e_map[i * w + arg_min] = remove[i * w + arg_min] ? SPECIAL_E : kernel(i, arg_min);
      }
    }
//    output_e_map(e_map, w, h, "e_map.png");
    Pic ret((u32) nw, (u32) h);
    for (i32 i = 0; i < h; ++i) {
      for (i32 j = 0; j < nw; ++j) {
        ret[i][j] = new_pic[i * pad_w + j];
      }
    }
    return ret;
  }

  Pic row_carve(f32 fac) const {
    return rotate().col_carve(fac).rotate();
  }

  Pic row_extend(f32 fac) const {
    return rotate().col_extend(fac).rotate();
  }
};

#include <chrono>

using namespace std::chrono;
auto now = high_resolution_clock::now;

i32 main(i32 argc, i8 **argv) {
  cmdline::parser parser;
  parser.add<std::string>("type", 't', "which process to exec[carve, extend, remove, protect]", true, "",
                          cmdline::oneof(std::string("carve"), std::string("extend"), std::string("remove"),
                                         std::string("protect")));
  parser.add<std::string>("input", 'i', "path of input .png file", true);
  parser.add<std::string>("output", 'o', "path of output .png file", true);
#if NEED_SEAM
  parser.add<std::string>("seam", 's', "path of output seam info .png file", true);
#endif
  parser.add<f32>("w", 'w', "the factor on w axis", false, 1.0);
  parser.add<f32>("h", 'h', "the factor on h axis", false, 1.0);
  parser.parse_check(argc, argv);
  std::string type = parser.get<std::string>("type");
  std::string input = parser.get<std::string>("input");
  std::string output = parser.get<std::string>("output");
#if NEED_SEAM
  std::string seam = parser.get<std::string>("seam");
#endif
  f32 fac_w = parser.get<f32>("w");
  f32 fac_h = parser.get<f32>("h");

  Pic pic(input.c_str());

  if (type == "carve") {
    if (fac_w <= 0 || fac_w > 1 || fac_h <= 0 || fac_h > 1) {
      printf("invalid w/h for carve: w=%f h=%f\n", fac_w, fac_h);
      exit(1);
    }
#if NEED_SEAM
    seam::init(pic.w, pic.h);
    memcpy(seam::col.data(), pic.a.get(), 3 * pic.w * pic.h);
#endif
    if (fac_w != 1) {
      auto beg = now();
      pic = pic.col_carve(fac_w);
      printf("%f\n", duration<f32>(now() - beg).count());
    }
    if (fac_h != 1) {
#if NEED_SEAM
      seam::rotate_idx();
      pic = pic.row_carve(fac_h);
      std::swap(seam::w, seam::h);
#else
      pic = pic.row_carve(fac_h);
#endif
    }
  } else if (type == "extend") {
    if (fac_w < 1 || fac_h < 1) {
      printf("invalid w/h for extend: w=%f h=%f\n", fac_w, fac_h);
      exit(1);
    }
#if NEED_SEAM
    u32 ext_w = u32(pic.w * fac_w), ext_h = u32(pic.h * fac_h);
    seam::init(ext_w, ext_h);
    for (u32 i = 0; i < pic.h; ++i) {
      memcpy(&seam::col[i * ext_w], &pic.a[i * 3 * pic.w], 3 * pic.w);
    }
#endif
    if (fac_w != 1) {
      pic = pic.col_extend(fac_w);
    }
    if (fac_h != 1) {
#if NEED_SEAM
      seam::rotate_col();
      pic = pic.row_extend(fac_h);
      seam::rotate_col();
#else
      pic = pic.row_extend(fac_h);
#endif
    }
  } else {
    if (fac_w <= 0 || fac_w >= 1) {
      printf("invalid w for remove/protect: w=%f\n", fac_w);
      exit(1);
    }
#if NEED_SEAM
    seam::init(pic.w, pic.h);
    memcpy(seam::col.data(), pic.a.get(), 3 * pic.w * pic.h);
#endif
    Rect r{};
    FILE *py = popen((std::string("python select_px.py ") + input).c_str(), "r");
    fscanf(py, "%d %d %d %d", &r.x1, &r.y1, &r.x2, &r.y2);
    pic = pic.rect(fac_w, r, type == "remove");
  }
  pic.output(output.c_str());
#if NEED_SEAM
  assert(lodepng_encode_file(seam.c_str(), (u8 *) (seam::col.data()), seam::w, seam::h, LCT_RGB, 8) == 0);
#endif
}
