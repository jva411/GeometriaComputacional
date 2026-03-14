use std::{cell::RefCell, rc::Rc};

use glam::Vec3;
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::object::{Object, ObjectType}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform}};

pub struct Sphere {
  pub id: Uuid,
  pub name: String,

  pub radius: f32,
  pub subdivisions: u32,

  pub transform: Transform,
  pub material: Material,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,

  _vertices: Vec<Vec3>,
  indices: Vec<usize>,
}

impl Sphere {
  pub fn new(name: String, radius: f32, subdivisions: u32) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let face_normals = [
      Vec3::X,
      Vec3::NEG_X,
      Vec3::Y,
      Vec3::NEG_Y,
      Vec3::Z,
      Vec3::NEG_Z,
    ];

    let resolution = subdivisions + 1;
    for &normal in face_normals.iter() {
      let axis_a = if normal.y.abs() < 0.99 {
        Vec3::Y.cross(normal).normalize()
      } else {
        Vec3::X.cross(normal).normalize()
      };
      let axis_b = normal.cross(axis_a).normalize();

      let start_index = vertices.len() as u32 / 2;

      for y in 0..=resolution {
        for x in 0..=resolution {
          let x_percent = x as f32 / resolution as f32;
          let y_percent = y as f32 / resolution as f32;

          let point_on_cube = normal + (axis_a * (2.0 * x_percent - 1.0)) + (axis_b * (2.0 * y_percent - 1.0));
          let point_on_sphere = point_on_cube.normalize();
          vertices.push(point_on_sphere * radius);
          vertices.push(point_on_sphere);
        }
      }

      for y in 0..resolution {
        for x in 0..resolution {
          let i = start_index + x + y * (resolution + 1);

          let bottom_left = i;
          let bottom_right = i + 1;
          let top_left = i + (resolution + 1);
          let top_right = i + (resolution + 1) + 1;

          indices.push(bottom_left as usize);
          indices.push(bottom_right as usize);
          indices.push(top_left as usize);

          indices.push(top_left as usize);
          indices.push(bottom_right as usize);
          indices.push(top_right as usize);
        }
      }
    }

    vao.add_attribute(0, 6 * SIZE_F32, 0);
    vao.add_attribute(1, 6 * SIZE_F32, 3 * SIZE_F32);

    let flat_data: Vec<f32> = vertices.iter().flat_map(|v| v.to_array()).collect();
    vbo.send_data(&flat_data);
    ebo.send_data(&indices.iter().map(|i| *i as u32).collect::<Vec<u32>>());

    return Self {
      id: Uuid::new_v4(),
      name,
      radius,
      subdivisions,
      transform: Transform::default(),
      material: Material::default(),
      vao,
      vbo,
      ebo,
      _vertices: vertices,
      indices: indices.iter().map(|i| *i as usize).collect(),
    };
  }
}

impl Object for Sphere {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Sphere }

  fn tick(&mut self) { }

  fn draw(&self, program: &Program, base_transform: Option<Transform>) {
    self.vao.bind();
    self.vbo.bind();
    self.ebo.bind();

    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };
    model_transform.send_to_program(&program);
    self.material.send_to_program(&program);

    unsafe {
      gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
    }
  }

  fn clone(&self) -> Self {
    let mut clone = Self::new(self.name.clone(), self.radius, self.subdivisions);
    clone.transform = self.transform.clone();
    clone.material = self.material.clone();
    return clone;
  }

  fn ray_intersection(&self, _ray: Ray) -> Option<f32> {
    unimplemented!()
  }
}

implement_transformable!(Sphere);
