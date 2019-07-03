from math import *

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


def rgb(l):
    r = 0.0
    g = 0.0
    b = 0.0
    if ((l >= 400.0) and (l < 410.0)):
        t = (l - 400.0) / (410.0 - 400.0)
        r = +(0.33 * t) - (0.20 * t * t)
    elif ((l >= 410.0) and (l < 475.0)):
        t = (l - 410.0) / (475.0 - 410.0)
        r = 0.14 - (0.13 * t * t)
    elif ((l >= 545.0) and (l < 595.0)):
        t = (l - 545.0) / (595.0 - 545.0)
        r = +(1.98 * t) - (t * t)
    elif ((l >= 595.0) and (l < 650.0)):
        t = (l - 595.0) / (650.0 - 595.0)
        r = 0.98 + (0.06 * t) - (0.40 * t * t)
    elif ((l >= 650.0) and (l < 700.0)):
        t = (l - 650.0) / (700.0 - 650.0)
        r = 0.65 - (0.84 * t) + (0.20 * t * t)

    if ((l >= 415.0) and (l < 475.0)):
        t = (l - 415.0) / (475.0 - 415.0)
        g = +(0.80 * t * t)
    elif ((l >= 475.0) and (l < 590.0)):
        t = (l - 475.0) / (590.0 - 475.0)
        g = 0.8 + (0.76 * t) - (0.80 * t * t)
    elif ((l >= 585.0) and (l < 639.0)):
        t = (l - 585.0) / (639.0 - 585.0)
        g = 0.84 - (0.84 * t)

    if ((l >= 400.0) and (l < 475.0)):
        t = (l - 400.0) / (475.0 - 400.0)
        b = +(2.20 * t) - (1.50 * t * t)
    elif ((l >= 475.0) and (l < 560.0)):
        t = (l - 475.0) / (560.0 - 475.0)
        b = 0.7 - (t) + (0.30 * t * t)

    return r, g, b


template = """    world.objs.push(Object {
      geo: Geo::Sphere(pe.ss[%d].s),
      color: Color::RGB(Vec3(%f, %f, %f)),
      texture: Texture::Refractive,
    });"""

mx = 5
mz = 5
rad = 0.9
idx = 0

for k in range(4):
    h = k * rad * sqrt(2)
    off = -rad * 3 - k * rad
    for i in range(4 - k):
        for j in range(4 - k):
            r, g, b = colors[int(idx / 30 * len(colors))]
            # r, g, b = rgb(401 + (700 - 401) * idx / 30)
            r = max(r, 0.3)
            g = max(g, 0.3)
            b = max(b, 0.3)

            x = mx + i * 2 * rad - off
            y = rad + h
            z = mz + j * 2 * rad - off
            idx += 1
            print(template % (idx - 1, r, g, b))

template = """pe.ss.push(MovingSphere { s: Sphere { c: Vec3(%f, %f, %f), r: %f }, v: Vec3(0.0, 3.1622776601683795, 0.0), m: 1.0 });"""

idx = 0
for k in range(4):
    h = k * rad * sqrt(2)
    off = -rad * 3 + k * rad
    for i in range(4 - k):
        for j in range(4 - k):
            r, g, b = colors[int(idx / 30 * len(colors))]
            # r, g, b = rgb(401 + (700 - 401) * idx / 30)
            r = max(r, 0.3)
            g = max(g, 0.3)
            b = max(b, 0.3)

            x = mx + i * 2 * rad + off
            y = rad + h
            z = mz + j * 2 * rad + off
            idx += 1
            print(template % (x, y+ 0.01, z , rad))


# for final rb
# template = """    {
#       Vec3 oc = Vec3{%f, %f, %f} - ray.o;
#       f32 b = oc.dot(ray.d);
#       f32 det = b * b - oc.len2() + %f * %f;
#       if (det > 0.0f) {
#         f32 sq_det = sqrtf(det);
#         f32 t = b - sq_det > EPS ? b - sq_det : b + sq_det > EPS ? b + sq_det : 0.0f;
#         if (t and t < res.t) {
#           res.t = t;
#           res.norm = (ray.o + ray.d * t - Vec3{%f, %f, %f}).norm();
#           res.text = 2;
#           res.col = Vec3{%f, %f, %f};
#         }
#       }
#     }"""
#
# mx = -5
# mz = 6
# rad = 0.9
# idx = 0

# for k in range(4):
#     h = k * rad * sqrt(2)
#     off = -rad * 3 - k * rad
#     for i in range(4 - k):
#         for j in range(4 - k):
#             r, g, b = colors[int(idx / 30 * len(colors))]
#             # r, g, b = rgb(401 + (700 - 401) * idx / 30)
#             r = max(r, 0.3)
#             g = max(g, 0.3)
#             b = max(b, 0.3)
#
#             x = mx + i * 2 * rad - off
#             y = rad + h
#             z = mz + j * 2 * rad - off
#             idx += 1
#             print(template % (x, y, z, rad, rad, x, y, z, r, g, b))
