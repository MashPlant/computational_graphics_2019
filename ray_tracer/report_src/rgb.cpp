#include <cmath>
#include <cstdio>
#include <vector>
#include <cstdlib>

using f32 = float;
using u32 = unsigned;
using u8 = unsigned char;

struct Vec3 {
  f32 x, y, z;

  void operator+=(const Vec3 &rhs) {
    x += rhs.x, y += rhs.y, z += rhs.z;
  }

  Vec3 operator*(f32 rhs) const {
    return {x * rhs, y * rhs, z * rhs};
  }
};

// // RGB <0,1> <- lambda l <400,700> [nm]
void spectral_color(f32 &r, f32 &g, f32 &b, f32 l) {
  f32 t;
  r = 0.0;
  g = 0.0;
  b = 0.0;
  if ((l >= 400.0) && (l < 410.0)) {
    t = (l - 400.0) / (410.0 - 400.0);
    r = +(0.33 * t) - (0.20 * t * t);
  } else if ((l >= 410.0) && (l < 475.0)) {
    t = (l - 410.0) / (475.0 - 410.0);
    r = 0.14 - (0.13 * t * t);
  } else if ((l >= 545.0) && (l < 595.0)) {
    t = (l - 545.0) / (595.0 - 545.0);
    r = +(1.98 * t) - (t * t);
  } else if ((l >= 595.0) && (l < 650.0)) {
    t = (l - 595.0) / (650.0 - 595.0);
    r = 0.98 + (0.06 * t) - (0.40 * t * t);
  } else if ((l >= 650.0) && (l < 700.0)) {
    t = (l - 650.0) / (700.0 - 650.0);
    r = 0.65 - (0.84 * t) + (0.20 * t * t);
  }
  if ((l >= 415.0) && (l < 475.0)) {
    t = (l - 415.0) / (475.0 - 415.0);
    g = +(0.80 * t * t);
  } else if ((l >= 475.0) && (l < 590.0)) {
    t = (l - 475.0) / (590.0 - 475.0);
    g = 0.8 + (0.76 * t) - (0.80 * t * t);
  } else if ((l >= 585.0) && (l < 639.0)) {
    t = (l - 585.0) / (639.0 - 585.0);
    g = 0.84 - (0.84 * t);
  }
  if ((l >= 400.0) && (l < 475.0)) {
    t = (l - 400.0) / (475.0 - 400.0);
    b = +(2.20 * t) - (1.50 * t * t);
  } else if ((l >= 475.0) && (l < 560.0)) {
    t = (l - 475.0) / (560.0 - 475.0);
    b = 0.7 - (t) + (0.30 * t * t);
  }
}

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

const u32 W = 1200, H = 400;
Vec3 output[H][W];

namespace phy {
  const f32 H = 6.62607004e-34;
  const f32 K = 1.38064852e-23;
  const f32 T = 5778.0;
  const f32 C = 299792458.0;
  const f32 B = 2.897772917e-3;
}

f32 planck(f32 l) {
  return 2.0 * phy::H * phy::C * phy::C / (powf(l, 5) * (expf((phy::H * phy::C) / (phy::K * phy::T * l)) - 1));
}

int main() {
  for (u32 i = 0; i < H; ++i) {
    for (u32 j = 1; j < W; ++j) {
      spectral_color(output[i][j].x, output[i][j].y, output[i][j].z, 400 + 300 * (f32(j) / W));
    }
  }
  output_png(&output[0][0], W, H, "rgb.png");

//  using namespace phy;
//  f32 max = phy::B / phy::T, max_i = planck(max);
//  Vec3 acc{};
//  for (u32 i = 0; i < W; ++i) {
//    f32 fac = 1;//planck(4e-7f + 3e-7f * (f32(i) / W)) / max_i;
//    printf("%f ", fac);
//    acc += output[0][i] * (1/fac);
//  }
//  puts("");
//  printf("%.10f %.10f\n", max, max_i);
//  printf("%f %f %f\n", acc.x, acc.y, acc.z);
}