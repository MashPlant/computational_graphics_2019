#include <cassert>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <vector>

using f32 = float;
using std::size_t;
using std::vector;

struct Vec3 {
  f32 x, y, z;

  constexpr Vec3 operator+(const Vec3 &rhs) const {
    return {x + rhs.x, y + rhs.y, z + rhs.z};
  }

  constexpr Vec3 operator-(const Vec3 &rhs) const {
    return {x - rhs.x, y - rhs.y, z - rhs.z};
  }

  constexpr Vec3 operator*(f32 rhs) const {
    return {x * rhs, y * rhs, z * rhs};
  }

  constexpr Vec3 operator/(f32 rhs) const { return *this * (1.0f / rhs); }

  constexpr Vec3 operator-() const { return {-x, -y, -z}; }

  void operator+=(const Vec3 &rhs) { x += rhs.x, y += rhs.y, z += rhs.z; }

  void operator-=(const Vec3 &rhs) { x -= rhs.x, y -= rhs.y, z -= rhs.z; }

  void operator*=(f32 rhs) { x *= rhs, y *= rhs, z *= rhs; }

  void operator/=(f32 rhs) { *this *= 1.0f / rhs; }

  constexpr f32 len2() const { return dot(*this); }

  constexpr f32 len() const { return std::sqrt(len2()); }

  constexpr Vec3 norm() const { return *this / len(); }

  constexpr Vec3 orthogonal_unit() const {
    return std::abs(y) != 1.0f ? Vec3{z, 0.0f, -x}.norm()
                               : Vec3{0.0f, z, -y}.norm();
  }

  constexpr Vec3 schur(const Vec3 &rhs) const {
    return {x * rhs.x, y * rhs.y, z * rhs.z};
  }

  constexpr f32 dot(const Vec3 &rhs) const {
    return x * rhs.x + y * rhs.y + z * rhs.z;
  }

  constexpr Vec3 cross(const Vec3 &rhs) const {
    return {y * rhs.z - z * rhs.y, z * rhs.x - x * rhs.z,
            x * rhs.y - y * rhs.x};
  }
};

struct Sphere {
  Vec3 c;
  f32 r;
};

#define let auto

struct MovingSphere : Sphere {
  Vec3 v;
  f32 m;

  MovingSphere(Sphere s, Vec3 v, f32 m) : Sphere(s), v(v), m(m) {}

  void do_collision(MovingSphere &other) {
    let &self = *this;
    let before_e = self.m * self.v.len2() + other.m * other.v.len2();
    let before_p = self.v * self.m + other.v * other.m;
    let dir = self.c - other.c;
    let v = other.v - self.v;
    let vo = dir * dir.dot(v) / dir.len2();
    let vo1 = vo * (2.0 * other.m) / (self.m + other.m);
    let vo2 = vo * (other.m - self.m) / (self.m + other.m);
    self.v += vo1;
    other.v += vo2 - vo;
    let after_e = self.m * self.v.len2() + other.m * other.v.len2();
    let after_p = self.v * self.m + other.v * other.m;

    // if (!(fabs(after_e - before_e) < 1e-3)) {
    //   // printf("%f %f %f\n", );
    //   // printf("%f %f %f\n", v);
    //   printf("%f %f\n", before_e, after_e);
    //   printf("%f %f %f\n", v.x, v.y, v.z);
    // }
    assert(fabs(after_e - before_e) < 1e-1);
    assert((after_p - before_p).len2() < 1e-1);
  }

  void gravity(MovingSphere &other, f32 G, f32 step) {
    let &self = *this;
    let dir = self.c - other.c;
    let r2 = dir.len2();
    let r3 = r2 * std::sqrt(r2);
    let common = dir * G * step / r3;
    self.v += -common * other.m;
    other.v += common * self.m;
  }

  void collision(MovingSphere &other) {
    let &self = *this;
    if ((other.c - self.c).len2() < (self.r + other.r) * (self.r + other.r)) {
      self.do_collision(other);
    }
  }
};

struct PhysicsEmulator {
  vector<MovingSphere> ss;
  Vec3 g;
  f32 G, step, bound[3][2];

  void next() {
    let &self = *this;
    for (size_t i = 0; i < self.ss.size(); ++i) {
      let s = self.ss[i]; // copy here to pass borrow check
      for (size_t j = i + 1; j < self.ss.size(); ++j) {
        s.gravity(self.ss[j], self.G, self.step);
        s.collision(self.ss[j]);
      }
      self.ss[i] = s;
    }
    for (let &s : self.ss) {
      s.v += self.g * self.step;
      if (s.c.x - s.r <= self.bound[0][0] || s.c.x + s.r >= self.bound[0][1]) {
        s.v.x = -s.v.x;
      }
      if (s.c.y - s.r <= self.bound[1][0] || s.c.y + s.r >= self.bound[1][1]) {
        s.v.y = -s.v.y;
      }
      if (s.c.z - s.r <= self.bound[2][0] || s.c.z + s.r >= self.bound[2][1]) {
        s.v.z = -s.v.z;
      }
      let v = s.v;
      s.c += v * self.step;
      // printf("%f %f %f\n", s.c.x, s.c.y, s.c.z);
    }
  }
};