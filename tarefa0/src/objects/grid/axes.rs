use glam::Vec3;

use crate::{objects::{object::Object, primitives::cylinder::Cylinder}, opengl::{program::Program, vao::VAO, vbo::VBO}, utils::{ray::Ray, transform::Transform}};

pub struct Axes {
  pub vao: VAO,
  pub vbo: VBO,
  vertex_count: i32,

  pub x_cylinder: Cylinder,
  pub y_cylinder: Cylinder,
  pub z_cylinder: Cylinder,
}

const AXES_VERTICES: [f32; 36] = [
//   X,   Y,   Z,     R,   G,   B
   0.0, 0.0, 0.0,   1.0, 0.0, 0.0, // Axis X
   1.0, 0.0, 0.0,   1.0, 0.0, 0.0,

   0.0, 0.0, 0.0,   0.0, 1.0, 0.0, // Axis Y
   0.0, 1.0, 0.0,   0.0, 1.0, 0.0,

   0.0, 0.0, 0.0,   0.0, 0.0, 1.0, // Axis Z
   0.0, 0.0, 1.0,   0.0, 0.0, 1.0,
];

#[allow(dead_code)]
impl Axes {
  pub fn new() -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    vao.bind();
    vbo.bind();

    vbo.send_data(&AXES_VERTICES);
    let vertex_count = 6;

    let stride = (6 * std::mem::size_of::<f32>()) as u32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, (3 * std::mem::size_of::<f32>()) as u32);

    let mut x_cylinder = Cylinder::new("".to_string(), 0.1, 1.0, 30);
    let mut y_cylinder = Cylinder::new("".to_string(), 0.1, 1.0, 30);
    let mut z_cylinder = Cylinder::new("".to_string(), 0.1, 1.0, 30);

    x_cylinder.transform.translate3f(0.5, 0.0, 0.0);
    y_cylinder.transform.translate3f(0.0, 0.5, 0.0);
    z_cylinder.transform.translate3f(0.0, 0.0, 0.5);

    x_cylinder.transform.add_roll(-90.0f32);
    z_cylinder.transform.add_pitch(90.0f32);


    Self {
      vao,
      vbo,
      vertex_count,

      x_cylinder,
      y_cylinder,
      z_cylinder,
    }
  }

  pub fn draw(&self, program: &Program, base_transform: Option<Transform>) {
    self.vao.bind();
    self.vbo.bind();

    let transform = match base_transform {
      Some(transform) => transform,
      None => Transform::default(),
    };
    transform.send_to_program(program);

    unsafe {
      gl::LineWidth(3.0);
      gl::DrawArrays(gl::LINES, 0, self.vertex_count);
      gl::LineWidth(1.0);
    }
  }

  pub fn ray_intersection(&self, ray: Ray, transform: &Transform) -> Option<Vec3> {
    let local_ray = ray.transform_inverse(transform);

    let x = self.x_cylinder.ray_intersection(local_ray.transform_inverse(&self.x_cylinder.transform));
    let y = self.y_cylinder.ray_intersection(local_ray.transform_inverse(&self.y_cylinder.transform));
    let z = self.z_cylinder.ray_intersection(local_ray.transform_inverse(&self.z_cylinder.transform));

    let mut has_intersection = false;
    let mut result = Vec3::ZERO;

    if let Some(..) = x {
      has_intersection = true;
      result.x = 1.0;
    }
    if let Some(..) = y {
      has_intersection = true;
      result.y = 1.0;
    }
    if let Some(..) = z {
      has_intersection = true;
      result.z = 1.0;
    }

    return if has_intersection { Some(result) } else { None };
  }
}

impl Clone for Axes {
  fn clone(&self) -> Self {
    let clone = Self::new();
    return clone;
  }
}
