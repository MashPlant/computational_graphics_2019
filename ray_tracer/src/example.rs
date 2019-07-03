extern crate ray_tracer;

use ray_tracer::{
  vec::Vec3,
  world::World,
  geo::*,
  material::*,
  mat44::Mat44,
  codegen::*,
  load,
};

fn main() {
  // /\y
  // |
  // |     * z
  // |
  // --------->x

  let world = World {
    objs: vec![
      Object { // left
        geo: Geo::InfPlane(InfPlane::new(Vec3(0.0, 0.0, 0.0), Vec3(1.0, 0.0, 0.0))),
        color: Color::RGB(Vec3(0.75, 0.25, 0.25)),
        texture: Texture::Diffuse,
      },
      Object { // right
        geo: Geo::InfPlane(InfPlane::new(Vec3(10.0, 0.0, 0.0), Vec3(1.0, 0.0, 0.0))),
        color: Color::RGB(Vec3(0.25, 0.25, 0.75)),
        texture: Texture::Diffuse,
      },
      Object { // back
        geo: Geo::InfPlane(InfPlane::new(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 1.0))),
        color: Color::RGB(Vec3(0.75, 0.75, 0.75)),
        texture: Texture::Diffuse,
      },
      Object { // front
        geo: Geo::InfPlane(InfPlane::new(Vec3(0.0, 0.0, 20.0), Vec3(0.0, 0.0, 1.0))),
        color: Color::RGB(Vec3(0.75, 0.75, 0.75)),
        texture: Texture::Diffuse,
      },
      Object { // bottom
        geo: Geo::InfPlane(InfPlane::new(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0))),
        color: Color::RGB(Vec3(0.75, 0.75, 0.75)),
        texture: Texture::Diffuse,
      },
      Object { // top
        geo: Geo::InfPlane(InfPlane::new(Vec3(0.0, 8.50, 0.0), Vec3(0.0, 1.0, 0.0))),
        color: Color::RGB(Vec3(0.75, 0.75, 0.75)),
        texture: Texture::Diffuse,
      },
      Object {
        geo: Geo::Mesh(load::mesh("resource/dragon.obj", Mat44::shift(3.0, 0.0, 5.0) * Mat44::scale(3.0, 3.0, 3.0)).unwrap()),
        color: Color::RGB(Vec3(1.0, 1.0, 1.0)),
        texture: Texture::Refractive,
      },
      Object {
        geo: Geo::Mesh(load::mesh("resource/dragon.obj", Mat44::shift(7.0, 0.0, 10.0) * Mat44::scale(3.0, 3.0, 3.0)).unwrap()),
        color: Color::RGB(Vec3(1.0, 1.0, 1.0)),
        texture: Texture::Refractive,
      },
    ],
    light: LightSource {
      geo: LightGeo::Circle(Circle::new(Vec3(5.0, 8.5 - 0.02, 5.0), Vec3(0.0, 1.0, 0.0), 1.75)),
      emission: Vec3(15.0, 15.0, 15.0),
    },
    env: Vec3::zero(),
    cam: Ray::new(Vec3(5.0, 5.2, 29.56), Vec3(0.0, -0.042612, -1.0)),
    w: 2048,
    h: 2048,
  };
  // 4 ways to render
  CodegenBase::new(PPMCodeGen::new(Vec3(0.0, 0.0, 0.0), Vec3(10.0, 8.5, 20.0))).gen(&world, "ppm_tracer.cpp");
  CodegenBase::new(CppCodegen).gen(&world, "pt_tracer.cpp");
  CodegenBase::new(CudaCodegen::new()).gen(&world, "pt_tracer.cu");
  let _ = world.path_tracing(8192).write("rs_image.png"); // may not work, haven't used for a long time...
}
