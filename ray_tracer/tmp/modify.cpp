#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "lodepng.h"

using u8 = unsigned char;
using u32 = unsigned;
using f32 = float;

u8 expand[20000000];

int main() {
  u32 w, h;
  u8 *in;
  lodepng_decode_file(&in, &w, &h, "bauhinia.png", LCT_RGB, 8);
  u8 gray = 255 * pow(0.75, 1 / 2.2);
  memset(expand, gray, sizeof expand);

  for (u32 i = 0; i < h; ++i) {
    for (u32 j = 0; j < w * 3; j += 3) {
      if (in[i * w * 3 + j] < 200 || in[i * w * 3 + j + 1] < 200 || in[i * w * 3 + j + 2] < 200) {
        in[i * w * 3 + j] = 255;
        in[i * w * 3 + j + 1] = 255;
        in[i * w * 3 + j + 2] = 255;
      } else {
        in[i * w * 3 + j] = 131;
        in[i * w * 3 + j + 1] = 185;
        in[i * w * 3 + j + 2] = 185;
      }
    }
  }
  // u32 w2 = w * 2, h2 = h * 2;
  const u32 RATIO = 2;
  for (u32 i = 0; i < h * RATIO; ++i) {
    for (u32 j = 0; j < w * RATIO * 3; j += 3) {
      expand[i * w * RATIO * 3 + j] = 131;
      expand[i * w * RATIO * 3 + j + 1] = 185;
      expand[i * w * RATIO * 3 + j + 2] = 185;
    }
  }
  for (u32 i = h * (RATIO - 1) / 2; i < h + h * (RATIO - 1) / 2; ++i) {
    memcpy(expand + (i * w * RATIO + w * (RATIO - 1) / 2) * 3, in + (i - h * (RATIO - 1) / 2) * w * 3, w * 3);
  }
  lodepng_encode_file("bauhinia_b.png", expand, w * RATIO, h * RATIO, LCT_RGB, 8);
}

// int main() {
//   const u32 STEP = 8;
//   const u8 R1 = 16, G1 = 63, B1 = 91;
//   const u8 R2 = 131, G2 = 200, B2 = 196;
//   const u8 RS = (R2 - R1) / (2 * STEP), GS = (G2 - G1) / (2 * STEP), BS = (B2 - B1) / (2 * STEP);
//   const u32 W = 2048, H = 1024;
//   for (u32 i = 0; i < H; ++i) {
//     for (u32 j = 0; j < W / 2; ++j) {
//       u32 idx = u32(f32(i) / H * STEP) + u32(f32(j) / (W / 2) * STEP);
//       expand[(i * W + W - 1 - j) * 3] = expand[(i * W + j) * 3] = std::min<u32>(255, R1 + RS * idx);
//       expand[(i * W + W - 1 - j) * 3 + 1] = expand[(i * W + j) * 3 + 1] = std::min<u32>(255, G1 + GS * idx);
//       expand[(i * W + W - 1 - j) * 3 + 2] = expand[(i * W + j) * 3 + 2] = std::min<u32>(255, B1 + BS * idx);
//     }
//   }
//   lodepng_encode_file("forest_grid.png", expand, W, H, LCT_RGB, 8);
// }

// int main() {
//   const u32 STEP = 8;
//   // const u8 R1 = 245, G1 = 107, B1 = 88;
//   // const u8 R2 = 255, G2 = 163, B2 = 114;
//   const u8 R1 = 214, G1 = 64, B1 = 33;
//   const u8 R2 = 255, G2 = 255, B2 = 130;
//   const u8 RS = (R2 - R1) / (2 * STEP), GS = (G2 - G1) / (2 * STEP), BS = (B2 - B1) / (2 * STEP);
//   const u32 W = 2048, H = 1024;
//   for (u32 i = 0; i < H; ++i) {
//     for (u32 j = 0; j < W / 2; ++j) {
//       u32 idx = u32(f32(i) / H * STEP) + u32(f32(j) / (W / 2) * STEP);
//       expand[(i * W + W - 1 - j) * 3] = expand[(i * W + j) * 3] = std::min<u32>(255, R1 + RS * idx);
//       expand[(i * W + W - 1 - j) * 3 + 1] = expand[(i * W + j) * 3 + 1] = std::min<u32>(255, G1 + GS * idx);
//       expand[(i * W + W - 1 - j) * 3 + 2] = expand[(i * W + j) * 3 + 2] = std::min<u32>(255, B1 + BS * idx);
//     }
//   }
//   lodepng_encode_file("desert_grid.png", expand, W, H, LCT_RGB, 8);
// }

// int main() {
//   u32 w, h;
//   u8 *in;
//   lodepng_decode_file(&in, &w, &h, "bauhinia.png", LCT_RGB, 8);
//   u8 gray = 255 * pow(0.75, 1 / 2.2);
//   memset(expand, gray, sizeof expand);

//   for (u32 i = 0; i < h; ++i) {
//     for (u32 j = 0; j < w * 3; j += 3) {
//       if (in[i * w * 3 + j] < 200 || in[i * w * 3 + j + 1] < 200 || in[i * w * 3 + j + 2] < 200) {
//         in[i * w * 3 + j] = 255;
//         in[i * w * 3 + j + 1] = 255;
//         in[i * w * 3 + j + 2] = 255;
//       } else {
//         in[i * w * 3 + j] =    255;
//         in[i * w * 3 + j + 1] = 200;
//         in[i * w * 3 + j + 2] = 160;
//       }
//     }
//   }
//   // u32 w2 = w * 2, h2 = h * 2;
//   const u32 RATIO = 2;
//   for (u32 i = 0; i < h * RATIO; ++i) {
//     for (u32 j = 0; j < w * RATIO * 3; j += 3) {
//       expand[i * w * RATIO * 3 + j] = 255;
//       expand[i * w * RATIO * 3 + j + 1] = 200;
//       expand[i * w * RATIO * 3 + j + 2] = 160;
//     }
//   }
//   for (u32 i = h * (RATIO - 1) / 2; i < h + h * (RATIO - 1) / 2; ++i) {
//     memcpy(expand + (i * w * RATIO + w * (RATIO - 1) / 2) * 3, in + (i - h * (RATIO - 1) / 2) * w * 3, w * 3);
//   }
//   lodepng_encode_file("bauhinia_r.png", expand, w * RATIO, h * RATIO, LCT_RGB, 8);
// }

// int main() {
//   u32 w, h;
//   u8 *in;
//   lodepng_decode_file(&in, &w, &h, "vase.png", LCT_RGB, 8);
//   u32 exp_h = h * 0.05;
//   memset(expand, 0xFF, sizeof expand);
//   memcpy(expand + w * exp_h * 3, in, w * h * 3);
//   lodepng_encode_file("vase_exp.png", expand, w, h + exp_h * 2, LCT_RGB, 8);
//   // printf("%d %d\n", w, h);
// }