#include <cmath>
#include <cstdlib>
#include <cstdio>
#include <cstring>

#ifdef __CUDACC__
#define DEVICE __device__
#define CONSTANT __constant__
#define SHARED __shared__
#define GLOBAL __global__
#else
#define DEVICE
#define CONSTANT
#define SHARED
#define GLOBAL
#endif
#define CUDA_BLOCK_SIZE 32

using u8 = unsigned char;
using u32 = unsigned;
using u64 = unsigned long long;
using f32 = float;

constexpr f32 EPS = 1.0f / 128.0f;
constexpr f32 PI = 3.14159265358979323846264338327950288f;

struct Vec3 {
  f32 x, y, z;

  DEVICE Vec3 operator+(const Vec3 &rhs) const {
    return {x + rhs.x, y + rhs.y, z + rhs.z};
  }

  DEVICE Vec3 operator-(const Vec3 &rhs) const {
    return {x - rhs.x, y - rhs.y, z - rhs.z};
  }

  DEVICE Vec3 operator+(f32 rhs) const {
    return {x + rhs, y + rhs, z + rhs};
  }

  DEVICE Vec3 operator-(f32 rhs) const {
    return {x - rhs, y - rhs, z - rhs};
  }

  DEVICE Vec3 operator*(f32 rhs) const {
    return {x * rhs, y * rhs, z * rhs};
  }

  DEVICE Vec3 operator/(f32 rhs) const {
    return *this * (1.0f / rhs);
  }

  DEVICE Vec3 operator-() const {
    return {-x, -y, -z};
  }

  DEVICE void operator+=(const Vec3 &rhs) {
    x += rhs.x, y += rhs.y, z += rhs.z;
  }

  DEVICE void operator-=(const Vec3 &rhs) {
    x -= rhs.x, y -= rhs.y, z -= rhs.z;
  }

  DEVICE void operator*=(f32 rhs) {
    x *= rhs, y *= rhs, z *= rhs;
  }

  DEVICE void operator/=(f32 rhs) {
    *this *= 1.0f / rhs;
  }

  DEVICE f32 len2() const {
    return dot(*this);
  }

  DEVICE f32 len() const {
    return sqrtf(len2());
  }

  DEVICE Vec3 norm() const {
    return *this / len();
  }

  DEVICE Vec3 orthogonal_unit() const {
    return fabsf(y) != 1.0f ? Vec3{z, 0.0f, -x}.norm() : Vec3{0.0f, z, -y}.norm();
  }

  DEVICE Vec3 schur(const Vec3 &rhs) const {
    return {x * rhs.x, y * rhs.y, z * rhs.z};
  }

  DEVICE f32 dot(const Vec3 &rhs) const {
    return x * rhs.x + y * rhs.y + z * rhs.z;
  }

  DEVICE Vec3 cross(const Vec3 &rhs) const {
    return {y * rhs.z - z * rhs.y, z * rhs.x - x * rhs.z, x * rhs.y - y * rhs.x};
  }

  DEVICE f32 operator[](u32 idx) const {
    return (&x)[idx];
  }

#ifdef __CUDACC__
  DEVICE static Vec3 from_float4(float4 f) {
    return {f.x, f.y, f.z};
  }
#endif
};

struct Vec2 {
  f32 x, y;

  DEVICE Vec2 operator+(const Vec2 &rhs) const {
    return {x + rhs.x, y + rhs.y};
  }

  DEVICE Vec2 operator-(const Vec2 &rhs) const {
    return {x - rhs.x, y - rhs.y};
  }

  DEVICE Vec2 operator*(f32 rhs) const {
    return {x * rhs, y * rhs};
  }

  DEVICE f32 dot(const Vec2 &rhs) const {
    return x * rhs.x + y * rhs.y;
  }

  DEVICE f32 len2() const {
    return dot(*this);
  }

  DEVICE f32 len() const {
    return sqrtf(len2());
  }

  DEVICE Vec3 to_vec3() const {
    return {x, y, 0.0f};
  }
};

DEVICE inline f32 mod1(f32 x) {
  x = fmodf(x, 1.0f);
  return x < 0.0f ? x + 1.0f : x;
}

struct Ray {
  Vec3 o, d;
};

struct XorShiftRNG {
  u32 seed;

  DEVICE XorShiftRNG(u32 seed) : seed(seed ? seed : 1) {}

  DEVICE f32 gen() {
    seed ^= seed << 13;
    seed ^= seed >> 17;
    seed ^= seed << 5;
    return seed * (1.0f / -1u);
  }
};

struct HitRes {
  f32 t;
  Vec3 norm;
  u32 text;
  Vec3 col;
};

#define BB_HIT_RAY_OUT(out_min, out_max, min, max, o, inv_d)                                     \
  ({                                                                                             \
    Vec3 __t0s = (min - o).schur(inv_d), t1s = (max - o).schur(inv_d);                           \
    out_min = fmaxf(fminf(__t0s.x, t1s.x), fmaxf(fminf(__t0s.y, t1s.y), fminf(__t0s.z, t1s.z))); \
    out_max = fminf(fmaxf(__t0s.x, t1s.x), fminf(fmaxf(__t0s.y, t1s.y), fmaxf(__t0s.z, t1s.z))); \
    0.0f < out_max && out_min < out_max;                                                         \
  })

#define BB_HIT_RAY(min, max, o, inv_d)                    \
  ({                                                      \
    f32 __t_min, __t_max;                                 \
    BB_HIT_RAY_OUT(__t_min, __t_max, min, max, o, inv_d); \
  })

// used for calculate triangle-ray hit
// http://jcgt.org/published/0005/03/03/
//struct TriMat {
//  f32 m00, m01, m02, m03;
//  f32 m10, m11, m12, m13;
//  f32 m20, m21, m22, m23;
//};

struct KDNode {
  Vec3 min, max;
  union {
    struct { // leaf, len = actual len | (1 << 31)
      u32 len;
      Vec3 pe[0];
//      TriMat ms[0]; // also store n & uv after ms
    };
    struct { // internal
      u32 ch1, sp_d;
      f32 sp;
    };
  };
};

// "short stack" algorithm
// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.83.2823&rep=rep1&type=pdf
// if col.x < 0.0, rt should contain color info(after ptr n)
DEVICE inline bool kd_node_hit(const KDNode * __restrict__ rt, const Ray &ray, HitRes &res, u32 text, const Vec3 &col) {
  struct {
    u32 off;
    f32 t_min, t_max;
  } stk[16];
  u32 top = 0;
  const char * __restrict__ rt_b = (const char *)rt;
  Vec3 inv_d{1.0f / ray.d.x, 1.0f / ray.d.y, 1.0f / ray.d.z};
  f32 root_min, root_max, t_min, t_max;
  const KDNode * __restrict__ x;
  bool push_down, hit = false;
  if (BB_HIT_RAY_OUT(root_min, root_max, rt->min, rt->max, ray.o, inv_d)) {
    t_max = root_min;
    while (t_max < root_max) {
      if (top == 0) {
        t_min = t_max;
        t_max = root_max;
        x = rt;
        push_down = true;
      } else {
        --top;
        t_min = stk[top].t_min;
        t_max = stk[top].t_max;
        x = (const KDNode *)(rt_b + stk[top].off);
        push_down = false;
      }
      while (BB_HIT_RAY(x->min, x->max, ray.o, inv_d)) {
        if (x->len >> 31) { // leaf
          u32 len = x->len & 0x7fffffff;
          const Vec3 * __restrict__ pe = x->pe, * __restrict__ n = pe + len;
          const Vec2 * __restrict__ uv = (const Vec2 *)(n + len);
          for (u32 i = 0; i < len; i += 3) {
            Vec3 p1 = pe[i], e1 = pe[i + 1], e2 = pe[i + 2];
            Vec3 p = ray.d.cross(e2);
            f32 det = e1.dot(p);
            f32 inv_det = 1.0f / det;
            Vec3 d = ray.o - p1;
            f32 u = d.dot(p) * inv_det;
            if (u < 0.0f || u > 1.0f) { continue; }
            Vec3 q = d.cross(e1);
            f32 v = ray.d.dot(q) * inv_det;
            if (v < 0.0f || u + v > 1.0f) { continue; };
            f32 t = e2.dot(q) * inv_det;
            if (t > EPS) {
              if (t < res.t) {
                res.t = t;
                res.norm = n[i] * (1.0f - u - v) + n[i + 1] * u + n[i + 2] * v;
                res.text = text;
                if (col.x < 0.0) {
                  res.col = (uv[i] * (1.0f - u - v) + uv[i + 1] * u + uv[i + 2] * v).to_vec3();
                } else {
                  res.col = col;
                }
                hit = true;
              }
              if (t < t_max) {
                return hit;
              }
            }
          }
          break;
        } else { // internal
          u32 sp_d = x->sp_d;
          f32 sp = x->sp;
          f32 t_sp = (sp - ray.o[sp_d]) / ray.d[sp_d];
          u32 fst = ((const char *)(x) - rt_b) + 24 + 12, snd = x->ch1;
          if (ray.d[sp_d] < 0.0) {
            u32 t = fst; fst = snd; snd = t;
          }
          if (t_sp <= t_min) {
            x = (const KDNode *) (rt_b + snd);
          } else if (t_sp >= t_max) {
            x = (const KDNode *) (rt_b + fst);
          } else {
            stk[top++] = {snd, t_sp, t_max};
            x = (const KDNode *) (rt_b + fst);
            t_max = t_sp;
            push_down = false;
          }
          if (push_down) {
            rt = x;
          }
        }
      }
    }
  }
  return false;
}

// assume polynomial coef is stored in reversed order
#define EVAL_BEZIER(ps, t, x, y) \
  do {                           \
    x = 0, y = 0;                \
    for (const auto &p : ps) {   \
      x = x * t + p[0];          \
      y = y * t + p[1];          \
    }                            \
  } while (0);

/*
Copyright (C) 2017 Milo Yip. All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

* Redistributions of source code must retain the above copyright notice, this
  list of conditions and the following disclaimer.

* Redistributions in binary form must reproduce the above copyright notice,
  this list of conditions and the following disclaimer in the documentation
  and/or other materials provided with the distribution.

* Neither the name of pngout nor the names of its
  contributors may be used to endorse or promote products derived from
  this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

inline void svpng(FILE *fp, u32 w, u32 h, const u8 *img, bool alpha) {
  static const u32 t[] = {0, 0x1db71064, 0x3b6e20c8, 0x26d930ac, 0x76dc4190, 0x6b6b51f4, 0x4db26158, 0x5005713c,
      /* CRC32 Table */    0xedb88320, 0xf00f9344, 0xd6d6a3e8, 0xcb61b38c, 0x9b64c2b0, 0x86d3d2d4, 0xa00ae278,
                          0xbdbdf21c};
  u32 a = 1, b = 0, c, p = w * (alpha ? 4 : 3) + 1, x, y, i;   /* ADLER-a, ADLER-b, CRC, pitch */
#define SVPNG_PUT(u) fputc(u, fp)
#define SVPNG_U8A(ua, l) for (i = 0; i < l; i++) SVPNG_PUT((ua)[i]);
#define SVPNG_U32(u) do { SVPNG_PUT((u) >> 24); SVPNG_PUT(((u) >> 16) & 255); SVPNG_PUT(((u) >> 8) & 255); SVPNG_PUT((u) & 255); } while(0)
#define SVPNG_U8C(u) do { SVPNG_PUT(u); c ^= (u); c = (c >> 4) ^ t[c & 15]; c = (c >> 4) ^ t[c & 15]; } while(0)
#define SVPNG_U8AC(ua, l) for (i = 0; i < l; i++) SVPNG_U8C((ua)[i])
#define SVPNG_U16LC(u) do { SVPNG_U8C((u) & 255); SVPNG_U8C(((u) >> 8) & 255); } while(0)
#define SVPNG_U32C(u) do { SVPNG_U8C((u) >> 24); SVPNG_U8C(((u) >> 16) & 255); SVPNG_U8C(((u) >> 8) & 255); SVPNG_U8C((u) & 255); } while(0)
#define SVPNG_U8ADLER(u) do { SVPNG_U8C(u); a = (a + (u)) % 65521; b = (b + a) % 65521; } while(0)
#define SVPNG_BEGIN(s, l) do { SVPNG_U32(l); c = ~0U; SVPNG_U8AC(s, 4); } while(0)
#define SVPNG_END() SVPNG_U32(~c)
  SVPNG_U8A("\x89PNG\r\n\32\n", 8);           /* Magic */
  SVPNG_BEGIN("IHDR", 13);                    /* IHDR chunk { */
  SVPNG_U32C(w);
  SVPNG_U32C(h);                              /*   Width & Height (8 bytes) */
  SVPNG_U8C(8);
  SVPNG_U8C(alpha ? 6 : 2);                   /*   Depth=8, Color=True color with/without alpha (2 bytes) */
  SVPNG_U8AC("\0\0\0", 3);                    /*   Compression=Deflate, Filter=No, Interlace=No (3 bytes) */
  SVPNG_END();                                /* } */
  SVPNG_BEGIN("IDAT", 2 + h * (5 + p) + 4);   /* IDAT chunk { */
  SVPNG_U8AC("\x78\1", 2);                    /*   Deflate block begin (2 bytes) */
  for (y = 0; y < h; y++) {                   /*   Each horizontal line makes a block for simplicity */
    SVPNG_U8C(y == h - 1);                    /*   1 for the last block, 0 for others (1 byte) */
    SVPNG_U16LC(p);
    SVPNG_U16LC(~p);                          /*   Size of block in little endian and its 1's complement (4 bytes) */
    SVPNG_U8ADLER(0);                         /*   No filter prefix (1 byte) */
    for (x = 0; x < p - 1; x++, img++)
      SVPNG_U8ADLER(*img);                    /*   Image pixel data */
  }
  SVPNG_U32C((b << 16) | a);                  /*   Deflate block end with adler (4 bytes) */
  SVPNG_END();                                /* } */
  SVPNG_BEGIN("IEND", 0);
  SVPNG_END();                                /* IEND chunk {} */
}

inline void output_png(const Vec3 *o, u32 w, u32 h, const char *path) {
  auto to_u8 = [](f32 x) { return u8(powf(x < 0.0f ? 0.0f : x > 1.0f ? 1.0f : x, 1.0f / 2.2f) * 255.0f + 0.5f); };
  FILE *fp = fopen(path, "w");
  u8 *png = (u8 *) malloc(w * h * 3 * sizeof(u8));
  for (u32 i = h - 1, cnt = 0; ~i; --i) {
    for (u32 j = 0; j < w; ++j) {
      png[cnt] = to_u8(o[i * w + j].x);
      png[cnt + 1] = to_u8(o[i * w + j].y);
      png[cnt + 2] = to_u8(o[i * w + j].z);
      cnt += 3;
    }
  }
  svpng(fp, w, h, png, false);
  fclose(fp);
  free(png);
}

#define CUDA_CHECK_ERROR(fn) do { auto code = fn; if (code != cudaSuccess) exit((fprintf(stderr,"gpu error %s @%s @%d\n", cudaGetErrorString(code), __FUNCTION__, __LINE__), -1)); } while(false)

Vec3 trace(Ray ray, XorShiftRNG &rng) {
  Vec3 fac{1.0f, 1.0f, 1.0f};
  for (u32 _ = 0; _ < 20; ++_) {
    if (fac.len2() <= 1e-4) { return Vec3{}; }
    HitRes res{1e10};
    {
      f32 dot_d_n = ray.d.dot(Vec3{1, 0, 0});
      f32 t = (Vec3{0, 0, 0} - ray.o).dot(Vec3{1, 0, 0}) / dot_d_n;
      if (t > EPS && t < res.t) {
        res.t = t;
        res.norm = Vec3{1, 0, 0};
        res.text = 0;
        res.col = Vec3{0.75, 0.25, 0.25};
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{1, 0, 0});
      f32 t = (Vec3{10, 0, 0} - ray.o).dot(Vec3{1, 0, 0}) / dot_d_n;
      if (t > EPS && t < res.t) {
        res.t = t;
        res.norm = Vec3{1, 0, 0};
        res.text = 0;
        res.col = Vec3{0.25, 0.25, 0.75};
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 0, 1});
      f32 t = (Vec3{0, 0, 0} - ray.o).dot(Vec3{0, 0, 1}) / dot_d_n;
      if (t > EPS && t < res.t) {
        res.t = t;
        res.norm = Vec3{0, 0, 1};
        res.text = 0;
        res.col = Vec3{0.75, 0.75, 0.75};
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 0, 1});
      f32 t = (Vec3{0, 0, 20} - ray.o).dot(Vec3{0, 0, 1}) / dot_d_n;
      if (t > EPS && t < res.t) {
        res.t = t;
        res.norm = Vec3{0, 0, 1};
        res.text = 0;
        res.col = Vec3{0.75, 0.75, 0.75};
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 1, 0});
      f32 t = (Vec3{0, 0, 0} - ray.o).dot(Vec3{0, 1, 0}) / dot_d_n;
      if (t > EPS && t < res.t) {
        res.t = t;
        res.norm = Vec3{0, 1, 0};
        res.text = 0;
        res.col = Vec3{0.75, 0.75, 0.75};
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 1, 0});
      f32 t = (Vec3{0, 8.5, 0} - ray.o).dot(Vec3{0, 1, 0}) / dot_d_n;
      if (t > EPS && t < res.t) {
        res.t = t;
        res.norm = Vec3{0, 1, 0};
        res.text = 0;
        res.col = Vec3{0.75, 0.75, 0.75};
      }
    }
    {
      extern const KDNode _binary_mesh0_start;
      constexpr f32 PS[][2] = {{6.7000427, 13.2}, {-24.400116, -33.6}, {39.20011, 33.6}, {-28.000055, -33.6}, {7.000035, 42}, {2.799972, -33.6}, {-5.599986, 16.8}, {2.799996, 0}, {1.0000005, 0.8}, };
      constexpr f32 DER[][2] = {{53.60034, 105.6}, {-170.80081, -235.2}, {235.20067, 201.6}, {-140.00027, -168}, {28.00014, 168}, {8.399916, -100.8}, {-11.199972, 33.6}, {2.799996, 0}, };
      constexpr f32 SHIFT_X = 1.5, SHIFT_Z = 6.5;
      if (kd_node_hit(&_binary_mesh0_start, ray, res, ({ f32 p = rng.gen(); p < 0.8 ? 0 : p < 1 ? 1 : 2; }), Vec3{-1, 0, 0})) {
        f32 u = res.col.x * (2 * PI);
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
        }
        if (err < 0.01) {
          res.t = t;
          res.norm = Vec3{a00, 0, a20}.cross(Vec3{a01, a11, a21});
          u = fmaxf(fminf(u / (2 * PI), 0.9999), 0);
          v = fmaxf(fminf(v, 0.9999), 0);
          extern const Vec3 _binary_img0_start[];
          res.col = _binary_img0_start[u32(v * 1428) * 3016 + u32(u * 3016)];
        }
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, -1, 0});
      f32 t = (Vec3{0, 0.8, 5} - ray.o).dot(Vec3{0, -1, 0}) / dot_d_n;
      if (t > EPS && t < res.t) {
        Vec3 p = ray.o + ray.d * t - Vec3{0, 0.8, 5};
        f32 u = p.dot(Vec3{0.33333334, 0, 0});
        f32 v = p.dot(Vec3{0, 0, 0.33333334});
        if (0.0f < u && u < 1.0f && 0.0f < v && v < 1.0f) {
          res.t = t;
          res.norm = Vec3{0, -1, 0};
          res.text = 0;
          extern const Vec3 _binary_img1_start[];
          res.col = _binary_img1_start[u32(v * 600) * 600 + u32(u * 600)];
        }
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 0, -1});
      f32 t = (Vec3{0, 0.8, 5} - ray.o).dot(Vec3{0, 0, -1}) / dot_d_n;
      if (t > EPS && t < res.t) {
        Vec3 p = ray.o + ray.d * t - Vec3{0, 0.8, 5};
        f32 u = p.dot(Vec3{0.33333334, 0, 0});
        f32 v = p.dot(Vec3{0, -5, 0});
        if (0.0f < u && u < 1.0f && 0.0f < v && v < 1.0f) {
          res.t = t;
          res.norm = Vec3{0, 0, -1};
          res.text = 0;
          extern const Vec3 _binary_img1_start[];
          res.col = _binary_img1_start[u32(v * 600) * 600 + u32(u * 600)];
        }
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 0, -1});
      f32 t = (Vec3{0, 0.8, 8} - ray.o).dot(Vec3{0, 0, -1}) / dot_d_n;
      if (t > EPS && t < res.t) {
        Vec3 p = ray.o + ray.d * t - Vec3{0, 0.8, 8};
        f32 u = p.dot(Vec3{0.33333334, 0, 0});
        f32 v = p.dot(Vec3{0, -5, 0});
        if (0.0f < u && u < 1.0f && 0.0f < v && v < 1.0f) {
          res.t = t;
          res.norm = Vec3{0, 0, -1};
          res.text = 0;
          extern const Vec3 _binary_img1_start[];
          res.col = _binary_img1_start[u32(v * 600) * 600 + u32(u * 600)];
        }
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{-1, 0, 0});
      f32 t = (Vec3{3, 0.8, 5} - ray.o).dot(Vec3{-1, 0, 0}) / dot_d_n;
      if (t > EPS && t < res.t) {
        Vec3 p = ray.o + ray.d * t - Vec3{3, 0.8, 5};
        f32 u = p.dot(Vec3{0, -5, 0});
        f32 v = p.dot(Vec3{0, 0, 0.33333334});
        if (0.0f < u && u < 1.0f && 0.0f < v && v < 1.0f) {
          res.t = t;
          res.norm = Vec3{-1, 0, 0};
          res.text = 0;
          extern const Vec3 _binary_img1_start[];
          res.col = _binary_img1_start[u32(v * 600) * 600 + u32(u * 600)];
        }
      }
    }
    {
      Vec3 oc = Vec3{4.7, 1.65, 3.7} - ray.o;
      f32 b = oc.dot(ray.d);
      f32 det = b * b - oc.len2() + 1.65 * 1.65;
      if (det > 0.0f) {
        f32 sq_det = sqrtf(det);
        f32 t = b - sq_det > EPS ? b - sq_det : b + sq_det > EPS ? b + sq_det : 0.0f;
        if (t && t < res.t) {
          res.t = t;
          res.norm = (ray.o + ray.d * t - Vec3{4.7, 1.65, 3.7}).norm();
          res.text = 1;
          res.col = Vec3{0.999, 0.999, 0.999};
        }
      }
    }
    {
      Vec3 oc = Vec3{8, 1.65, 8} - ray.o;
      f32 b = oc.dot(ray.d);
      f32 det = b * b - oc.len2() + 1.65 * 1.65;
      if (det > 0.0f) {
        f32 sq_det = sqrtf(det);
        f32 t = b - sq_det > EPS ? b - sq_det : b + sq_det > EPS ? b + sq_det : 0.0f;
        if (t && t < res.t) {
          res.t = t;
          res.norm = (ray.o + ray.d * t - Vec3{8, 1.65, 8}).norm();
          res.text = 2;
          res.col = Vec3{0.999, 0.999, 0.999};
        }
      }
    }
    {
      f32 dot_d_n = ray.d.dot(Vec3{0, 1, 0});
      f32 t = (Vec3{5, 8.48, 8} - ray.o).dot(Vec3{0, 1, 0}) / dot_d_n;
      if (t > EPS && t < res.t && (ray.o + ray.d * t - Vec3{5, 8.48, 8}).len2() < 3.0625) {
        return fac.schur(Vec3{15, 15, 15});
      }
    }
    if (res.t == 1e10) { break; }
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
    }
  }
  return Vec3{};
}

const u32 W = 2048, H = 2048;
Vec3 output[W * H];

int main(int argc, char **args) {
  u32 ns = argc > 1 ? std::atoi(args[1]) : (puts("please specify #sample"), exit(-1), 0);
  u32 seed = argc > 2 ? std::atoi(args[2]) : (puts("please specify seed"), exit(-1), 0);
  seed = seed * 19260817 + 19660813;
  const char *out = argc > 3 ? args[3] : (puts("please specify out file"), exit(-1), nullptr);
  constexpr Ray cam{Vec3{5, 5.2, 29.56}, Vec3{0, -0.042573366, -0.9990933}};
  constexpr Vec3 cx{0.5135, 0, 0};
  constexpr Vec3 cy{0, 0.5130344, -0.021861423};
#pragma omp parallel for schedule(dynamic, 1)
  for (u32 y = 0; y < H; ++y) {
    fprintf(stderr, "\rrendering %5.2f%%", 100.0f * y / (H - 1));
    for (u32 x = 0; x < W; ++x) {
      u32 index = y * W + x;
      Vec3 sum{};
      XorShiftRNG rng{index + seed};
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
  fwrite(output, sizeof output, 1, fopen(out, "w"));
  // output_png(output, W, H, argc > 2 ? args[2] : "image.png");
}
/*
      Object {
        geo: Geo::RotateBezier(load::bezier_curve("tmp/vase.txt").unwrap().rotate_bezier(100, 100,
                                                                                         Vec3(1.5, 0.8, 6.5), 1.5)),
        color: load::texture("./tmp/vase.png", false).unwrap(),
        texture: Texture::Mixed { d_prob: 0.8, s_prob: 0.2 },
      },
      Object {
        geo: Geo::Rectangle(Rectangle::new(Vec3(0.0, 0.8, 5.0), Vec3(3.0, 0.0, 0.0), Vec3(0.0, 0.0, 3.0))),
        color: load::texture("./tmp/wood.png", false).unwrap(),
        texture: Texture::Diffuse,
      },
      Object {
        geo: Geo::Rectangle(Rectangle::new(Vec3(0.0, 0.8, 5.0), Vec3(3.0, 0.0, 0.0), Vec3(0.0, -0.2, 0.0))),
        color: load::texture("./tmp/wood.png", false).unwrap(),
        texture: Texture::Diffuse,
      },
      Object {
        geo: Geo::Rectangle(Rectangle::new(Vec3(0.0, 0.8, 8.0), Vec3(3.0, 0.0, 0.0), Vec3(0.0, -0.2, 0.0))),
        color: load::texture("./tmp/wood.png", false).unwrap(),
        texture: Texture::Diffuse,
      },
      Object {
        geo: Geo::Rectangle(Rectangle::new(Vec3(3.0, 0.8, 5.0), Vec3(0.0, -0.2, 0.0), Vec3(0.0, 0.0, 3.0))),
        color: load::texture("./tmp/wood.png", false).unwrap(),
        texture: Texture::Diffuse,
      },
      Object {
        geo: Geo::Sphere(Sphere { c: Vec3(4.7, 1.65, 3.7), r: 1.65 }),
        color: Color::RGB(Vec3(0.999, 0.999, 0.999)),
        texture: Texture::Specular,
      },
      Object {
        geo: Geo::Sphere(Sphere { c: Vec3(8.0, 1.65, 8.0), r: 1.65 }),
        color: Color::RGB(Vec3(0.999, 0.999, 0.999)),
        texture: Texture::Refractive,


0.666667 0.000000
0.900000 0.000000
1.000000 0.400000
1.000000 0.800000
1.000000 1.200000
0.833333 1.600000
0.666667 2.400000
0.500000 2.800000
1.000000 3.200000
*/