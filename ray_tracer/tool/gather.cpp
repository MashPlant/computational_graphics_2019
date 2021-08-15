#include <cmath>
#include <cstdio>
#include <cstdlib>

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

  DEVICE Vec3 operator+(const Vec3 &rhs) const { return {x + rhs.x, y + rhs.y, z + rhs.z}; }

  DEVICE Vec3 operator-(const Vec3 &rhs) const { return {x - rhs.x, y - rhs.y, z - rhs.z}; }

  DEVICE Vec3 operator+(f32 rhs) const { return {x + rhs, y + rhs, z + rhs}; }

  DEVICE Vec3 operator-(f32 rhs) const { return {x - rhs, y - rhs, z - rhs}; }

  DEVICE Vec3 operator*(f32 rhs) const { return {x * rhs, y * rhs, z * rhs}; }

  DEVICE Vec3 operator/(f32 rhs) const { return *this * (1.0f / rhs); }

  DEVICE Vec3 operator-() const { return {-x, -y, -z}; }

  DEVICE void operator+=(const Vec3 &rhs) { x += rhs.x, y += rhs.y, z += rhs.z; }

  DEVICE void operator-=(const Vec3 &rhs) { x -= rhs.x, y -= rhs.y, z -= rhs.z; }

  DEVICE void operator*=(f32 rhs) { x *= rhs, y *= rhs, z *= rhs; }

  DEVICE void operator/=(f32 rhs) { *this *= 1.0f / rhs; }

  DEVICE f32 len2() const { return dot(*this); }

  DEVICE f32 len() const { return sqrtf(len2()); }

  DEVICE Vec3 norm() const { return *this / len(); }

  DEVICE Vec3 orthogonal_unit() const { return fabsf(y) != 1.0f ? Vec3{z, 0.0f, -x}.norm() : Vec3{0.0f, z, -y}.norm(); }

  DEVICE Vec3 schur(const Vec3 &rhs) const { return {x * rhs.x, y * rhs.y, z * rhs.z}; }

  DEVICE f32 dot(const Vec3 &rhs) const { return x * rhs.x + y * rhs.y + z * rhs.z; }

  DEVICE Vec3 cross(const Vec3 &rhs) const { return {y * rhs.z - z * rhs.y, z * rhs.x - x * rhs.z, x * rhs.y - y * rhs.x}; }

  DEVICE f32 operator[](u32 idx) const { return (&x)[idx]; }

#ifdef __CUDACC__
  DEVICE static Vec3 from_float4(float4 f) { return {f.x, f.y, f.z}; }
#endif
};

inline void svpng(FILE *fp, u32 w, u32 h, const u8 *img, bool alpha) {
  static const u32 t[] = {0,
                          0x1db71064,
                          0x3b6e20c8,
                          0x26d930ac,
                          0x76dc4190,
                          0x6b6b51f4,
                          0x4db26158,
                          0x5005713c,
                          /* CRC32 Table */ 0xedb88320,
                          0xf00f9344,
                          0xd6d6a3e8,
                          0xcb61b38c,
                          0x9b64c2b0,
                          0x86d3d2d4,
                          0xa00ae278,
                          0xbdbdf21c};
  u32 a = 1, b = 0, c, p = w * (alpha ? 4 : 3) + 1, x, y, i; /* ADLER-a, ADLER-b, CRC, pitch */
#define SVPNG_PUT(u) fputc(u, fp)
#define SVPNG_U8A(ua, l) \
  for (i = 0; i < l; i++) SVPNG_PUT((ua)[i]);
#define SVPNG_U32(u)              \
  do {                            \
    SVPNG_PUT((u) >> 24);         \
    SVPNG_PUT(((u) >> 16) & 255); \
    SVPNG_PUT(((u) >> 8) & 255);  \
    SVPNG_PUT((u)&255);           \
  } while (0)
#define SVPNG_U8C(u)          \
  do {                        \
    SVPNG_PUT(u);             \
    c ^= (u);                 \
    c = (c >> 4) ^ t[c & 15]; \
    c = (c >> 4) ^ t[c & 15]; \
  } while (0)
#define SVPNG_U8AC(ua, l) \
  for (i = 0; i < l; i++) SVPNG_U8C((ua)[i])
#define SVPNG_U16LC(u)           \
  do {                           \
    SVPNG_U8C((u)&255);          \
    SVPNG_U8C(((u) >> 8) & 255); \
  } while (0)
#define SVPNG_U32C(u)             \
  do {                            \
    SVPNG_U8C((u) >> 24);         \
    SVPNG_U8C(((u) >> 16) & 255); \
    SVPNG_U8C(((u) >> 8) & 255);  \
    SVPNG_U8C((u)&255);           \
  } while (0)
#define SVPNG_U8ADLER(u)   \
  do {                     \
    SVPNG_U8C(u);          \
    a = (a + (u)) % 65521; \
    b = (b + a) % 65521;   \
  } while (0)
#define SVPNG_BEGIN(s, l) \
  do {                    \
    SVPNG_U32(l);         \
    c = ~0U;              \
    SVPNG_U8AC(s, 4);     \
  } while (0)
#define SVPNG_END() SVPNG_U32(~c)
  SVPNG_U8A("\x89PNG\r\n\32\n", 8); /* Magic */
  SVPNG_BEGIN("IHDR", 13);          /* IHDR chunk { */
  SVPNG_U32C(w);
  SVPNG_U32C(h); /*   Width & Height (8 bytes) */
  SVPNG_U8C(8);
  SVPNG_U8C(alpha ? 6 : 2);                 /*   Depth=8, Color=True color with/without alpha (2 bytes) */
  SVPNG_U8AC("\0\0\0", 3);                  /*   Compression=Deflate, Filter=No, Interlace=No (3 bytes) */
  SVPNG_END();                              /* } */
  SVPNG_BEGIN("IDAT", 2 + h * (5 + p) + 4); /* IDAT chunk { */
  SVPNG_U8AC("\x78\1", 2);                  /*   Deflate block begin (2 bytes) */
  for (y = 0; y < h; y++) {                 /*   Each horizontal line makes a block for simplicity */
    SVPNG_U8C(y == h - 1);                  /*   1 for the last block, 0 for others (1 byte) */
    SVPNG_U16LC(p);
    SVPNG_U16LC(~p);                                        /*   Size of block in little endian and its 1's complement (4 bytes) */
    SVPNG_U8ADLER(0);                                       /*   No filter prefix (1 byte) */
    for (x = 0; x < p - 1; x++, img++) SVPNG_U8ADLER(*img); /*   Image pixel data */
  }
  SVPNG_U32C((b << 16) | a); /*   Deflate block end with adler (4 bytes) */
  SVPNG_END();               /* } */
  SVPNG_BEGIN("IEND", 0);
  SVPNG_END(); /* IEND chunk {} */
}

inline void output_png(const Vec3 *o, u32 w, u32 h, const char *path) {
  auto to_u8 = [](f32 x) { return u8(powf(x < 0.0f ? 0.0f : x > 1.0f ? 1.0f : x, 1.0f / 2.2f) * 255.0f + 0.5f); };
  FILE *fp = fopen(path, "w");
  u8 *png = (u8 *)malloc(w * h * 3 * sizeof(u8));
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

const u32 W = 1024, H = 1024;
Vec3 output[W * H], tmp[W * H];

int main(int argc, char **argv) {
  for (u32 i = 1; i < argc; ++i) {
    FILE *f = fopen(argv[i], "r");
    fread(tmp, sizeof tmp, 1, f);
    for (u32 i = 0; i < W * H; ++i) {
      output[i] += tmp[i];
    }
    fclose(f);
  }
  for (u32 i = 0; i < W * H; ++i) {
    output[i] /= argc - 1;
  }
  output_png(output, W, H, "image.png");
}