#include <vector>
#include <mutex>

constexpr f32 ALPHA = 0.7f;

// generate low discrepancy random numbers using Halton sequence
// with div/mod const optimization
struct HaltonRNG {
  u64 mul;
  u32 val, shift;
  f32 inv;

  HaltonRNG(u32 prime) : val(prime), shift(63 - __builtin_clz(prime)), inv(1.0f / prime) {
    mul = ((u64) 1 << shift) / val;
    mul += (val * mul >> shift) != 1;
  }

  f32 gen(u32 x) const {
    f32 ret = 0.0, fac = inv;
    while (x) {
      u32 div = x * mul >> shift;
      ret += (x - val * div) * fac;
      x = div;
      fac *= inv;
    }
    return ret;
  }
};

const HaltonRNG hal[] = {
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109,
    113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211, 223, 227, 229, 233, 239,
    241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307, 311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379,
    383, 389, 397, 401, 409, 419, 421, 431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499
};

struct HitPoint {
  Vec3 fac, pos, norm, flux;
  f32 r2;
  u32 n; // n = N / ALPHA in the paper
  u32 idx;
};

struct HashGrid {
  Vec3 min, max;
  f32 inv_grid_size;
  std::vector<HitPoint> hps;
  std::vector<HitPoint *> pool;
  std::vector<u32> idx;
  std::mutex mu;

  u32 hash(u32 x, u32 y, u32 z) const {
    return ((x * 73856093) ^ (y * 19349663) ^ (z * 83492791)) % hps.size();
  }

  void rebuild(u32 w, u32 h) {
    fprintf(stderr, "building hash grid...\n");
    min = Vec3{1e10, 1e10, 1e10}, max = Vec3{-1e10, -1e10, -1e10};
    auto update = [this](const Vec3 &v) {
      min.x = fminf(v.x, min.x), min.y = fminf(v.y, min.y), min.z = fminf(v.z, min.z);
      max.x = fmaxf(v.x, max.x), max.y = fmaxf(v.y, max.y), max.z = fmaxf(v.z, max.z);
    };
    for (auto &hp : hps) {
      update(hp.pos);
    }
    fprintf(stderr, "hash grid min: %f %f %f\n", min.x, min.y, min.z);
    fprintf(stderr, "hash grid max: %f %f %f\n", max.x, max.y, max.z);
    Vec3 size = max - min;
    f32 rad = ((size.x + size.y + size.z) / 3.0) / ((w + h) / 2.0) * 2.0; // heuristic for initial radius
    fprintf(stderr, "init rad: %f\n", rad);
    min = Vec3{1e10, 1e10, 1e10}, max = Vec3{-1e10, -1e10, -1e10};
    for (auto &hp : hps) {
      hp.r2 = rad * rad;
      hp.n = 0;
      hp.flux = Vec3{};
      update(hp.pos - rad);
      update(hp.pos + rad);
    }
    inv_grid_size = 1.0 / (rad * 2.0); // make each grid cell two times larger than the initial radius
    idx.resize(hps.size() + 1);
    u32 all = 0;
    for (auto &hp : hps) {
      Vec3 min1 = (hp.pos - min - rad) * inv_grid_size;
      Vec3 max1 = (hp.pos - min + rad) * inv_grid_size;
      for (u32 z = u32(min1.z); z <= u32(max1.z); ++z) {
        for (u32 y = u32(min1.y); y <= u32(max1.y); ++y) {
          for (u32 x = u32(min1.x); x <= u32(max1.x); ++x) {
            ++idx[hash(x, y, z) + 1];
            ++all;
          }
        }
      }
    }
    for (u32 i = 1; i < idx.size(); ++i) {
      idx[i] += idx[i - 1];
    }
    std::vector<u32> idx_copy = idx;
    pool.resize(all);
    for (auto &hp : hps) {
      Vec3 min1 = (hp.pos - min - rad) * inv_grid_size;
      Vec3 max1 = (hp.pos - min + rad) * inv_grid_size;
      for (u32 z = u32(min1.z); z <= u32(max1.z); ++z) {
        for (u32 y = u32(min1.y); y <= u32(max1.y); ++y) {
          for (u32 x = u32(min1.x); x <= u32(max1.x); ++x) {
            pool[--idx_copy[hash(x, y, z) + 1]] = &hp;
          }
        }
      }
    }
    fprintf(stderr, "hash grid info: %d points, %d entry, mem = %.1fM\n", u32(hps.size()), all,
            (hps.size() * (sizeof(HitPoint) + sizeof(u32)) + all * sizeof(HitPoint *)) / 1e6f);
  }
} grid;
