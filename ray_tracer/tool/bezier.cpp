#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cmath>
#include <ctime>

using u32 = unsigned;
using f32 = float;
using f128 = long double;

const u32 N = 20;
f32 ps[N][2];
u32 cs[N][N];
f128 xas[N], yas[N];

f32 evalx(f32 x, u32 n) {
  f32 ret = 0;
  for (u32 i = n; ~i; --i) {
    ret = ret * x + f32(xas[i]);
  }
  return ret;
}

f32 evaly(f32 x, u32 n) {
  f32 ret = 0;
  for (u32 i = n; ~i; --i) {
    ret = ret * x + f32(yas[i]);
  }
  return ret;
}

f32 randf() {
  return (f32(rand() - RAND_MAX / 2) / RAND_MAX) * 2;
}

int main() {
  using ld = double;
  const ld bezier_div_x = 30;
  const ld bezier_div_y = 25;
  ld control_x[] = {20. / bezier_div_x, 27. / bezier_div_x, 30. / bezier_div_x, 30. / bezier_div_x, 30. / bezier_div_x,
                    25. / bezier_div_x, 20. / bezier_div_x, 15. / bezier_div_x, 30. / bezier_div_x};
  ld control_y[] = {0. / bezier_div_y, 0. / bezier_div_y, 10. / bezier_div_y, 20. / bezier_div_y, 30. / bezier_div_y,
                    40. / bezier_div_y, 60. / bezier_div_y, 70. / bezier_div_y, 80. / bezier_div_y};
  for (u32 i = 0; i < sizeof(control_x) / 8; ++i) {
    printf("%lf %lf\n", control_x[i], control_y[i]);
  }
}
//int main() {
//  srand(time(0));
//  f32 s0 = randf(), s1 = randf(), s2 = randf();
//  f32 a00 = randf(), a01 = randf(), a02 = randf(), b0 = s0 * a00 + s1 * a01 + s2 * a02;
//  f32 /*a10,*/ a11 = randf(), a12 = randf(), b1 = s1 * a11 + s2 * a12;
//  f32 a20 = randf(), a21 = randf(), a22 = randf(), b2 = s0 * a20 + s1 * a21 + s2 * a22;
//  {
//    f32 fac = a20 / a00;
//    a21 -= fac * a01, a22 -= fac * a02, b2 -= fac * b0;
//  }
//  {
//    f32 fac = a21 / a11;
//    a22 -= fac * a12, b2 -= fac * b1;
//  }
//  f32 x2 = b2 / a22;
//  f32 x1 = (b1 - x2 * a12) / a11;
//  f32 x0 = (b0 - x2 * a02 - x1 * a01) / a00;
//  printf("%f %f %f\n", x0, x1, x2);
//  printf("%f %f %f\n", s0, s1, s2);
//}
//int main() {
//  u32 n = 0;
//  while (~scanf("%f %f", &ps[n][0], &ps[n][1])) ++n;
//  --n; // now n is the len of xas &
//
//  printf("%d\n", n);
//  for (u32 i = 0; i <= n; ++i) {
//    printf("%f ", ps[i][0]);
//  }
//  puts("");
//
//  cs[0][0] = 1;
//  for (u32 i = 1; i <= n; ++i) {
//    cs[i][0] = 1;
//    for (u32 j = 1; j <= i; ++j) {
//      cs[i][j] = cs[i - 1][j] + cs[i - 1][j - 1];
//    }
//  }
////  for (u32 i = 1; i <= n; ++i, puts("")) {
////    for (u32 j = 0; j <= i; ++j) {
////      printf("%d ", cs[i][j]);
////    }
////  }
//  for (u32 i = 0; i <= n; ++i) {
//    f128 fac = f128(ps[i][0]) * cs[n][i];
//    for (u32 j = i; j <= n; ++j) {
//      f128 tmp = fac * cs[n - i][j - i];
//      xas[j] += ((j - i) & 1) ? -tmp : tmp;
//    }
//  }
//  for (u32 i = 0; i <= n; ++i) {
//    f128 fac = f128(ps[i][1]) * cs[n][i];
//    for (u32 j = i; j <= n; ++j) {
//      f128 tmp = fac * cs[n - i][j - i];
//      yas[j] += ((j - i) & 1) ? -tmp : tmp;
//    }
//  }
//
//  for (u32 i = 0; i <= n; ++i) {
//    printf("%Lf ", xas[i]);
//  }
//  puts("");
//  for (u32 i = 0; i <= n; ++i) {
//    printf("%Lf ", yas[i]);
//  }
//  puts("");
////  for (f32 x = -0.1; x < 1.15; x += 0.05) {
////    printf("%f %f\n", eval1(x, n), eval2(x, n));
////  }
////  puts("");
//
////  for (f32 x = 0; x < 1.005; x += 0.01) {
////    printf("%f %f\n", evalx(x, n), evaly(x, n));
////
////    f128 ansx = 0, ansy = 0;
////    for (u32 i = 0; i <= n; ++i) {
////      ansx += ps[i][0] * cs[n][i] * std::pow(x, i) * std::pow(1 - x, n - i);
////      ansy += ps[i][1] * cs[n][i] * std::pow(x, i) * std::pow(1 - x, n - i);
////    }
//////    printf("%f %f\n", (f32) ansx, (f32) ansy);
////  }
//  puts("");
//
//}
