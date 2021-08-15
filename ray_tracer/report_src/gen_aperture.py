colors = []


def range_ck(x): return 255 if x > 255 else 0 if x < 0 else x


r, g, b = 0, 0, 255
r_f, g_f, b_f = 0, 0, 1

while True:
    colors.append((r / 255, g / 255, b / 255))
    if (b == 255):
        g_f = 1
    if (g == 255):
        b_f = -1
    if (b == 0):
        r_f = +1
    if (r == 255):
        g_f = -1
    if (g == 0 and b == 0):
        r_f = -1
    if (r < 127 and g == 0 and b == 0):
        break
    r = range_ck(r + r_f)
    g = range_ck(g + g_f)
    b = range_ck(b + b_f)

template = """    {
      Vec3 oc = Vec3{1 + %f, 1.2, 2 + %f} - ray.o;
      f32 b = oc.dot(ray.d);
      f32 det = b * b - oc.len2() + 0.65 * 0.65;
      if (det > 0.0f) {
        f32 sq_det = sqrtf(det);
        f32 t = b - sq_det > EPS ? b - sq_det : b + sq_det > EPS ? b + sq_det : 0.0f;
        if (t && t < res.t) {
          res.t = t;
          res.norm = (ray.o + ray.d * t - Vec3{1 + %f, 1.2, 2 + %f}).norm();
          res.text = 2;
          res.col = Vec3{%f, %f, %f};
        }
      }
    }"""

step = 1
iter = 9

for i in range(iter):
    r, g, b = colors[int(i / iter * len(colors))]
    r = max(r, 0.3)
    g = max(g, 0.3)
    b = max(b, 0.3)

    print(template % (i * step, i * step, i * step, i * step, r, g, b))

for i in range(iter-1):
    r, g, b = colors[int(i / iter * len(colors))]
    r = max(r, 0.3)
    g = max(g, 0.3)
    b = max(b, 0.3)

    print(template % (i * step, (2 * (iter-1) - i) * step,
                      i * step, (2 * (iter-1) - i) * step, r, g, b))
