use std::{cell::RefCell, rc::Rc};

use glam::Vec3;
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::{geometry::points_cloud::PointsCloud, object::{Object, ObjectType}}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform}};

#[allow(dead_code)]
pub struct Cube {
  pub id: Uuid,
  pub name: String,

  pub transform: Transform,
  pub material: Material,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,
}

pub const VERTICES: [f32; (3 + 3) * 24] = [
  // X     Y    Z     Nx    Ny    Nz
  -0.5, -0.5, -0.5,     0.0,  0.0, -1.0,  // 0  Back
   0.5, -0.5, -0.5,     0.0,  0.0, -1.0,  // 1
   0.5,  0.5, -0.5,     0.0,  0.0, -1.0,  // 2
  -0.5,  0.5, -0.5,     0.0,  0.0, -1.0,  // 3

  -0.5, -0.5,  0.5,     0.0,  0.0,  1.0,  // 4  Front
  -0.5,  0.5,  0.5,     0.0,  0.0,  1.0,  // 5
   0.5,  0.5,  0.5,     0.0,  0.0,  1.0,  // 6
   0.5, -0.5,  0.5,     0.0,  0.0,  1.0,  // 7

  -0.5, -0.5, -0.5,    -1.0,  0.0,  0.0,  // 8  Left
  -0.5,  0.5, -0.5,    -1.0,  0.0,  0.0,  // 9
  -0.5,  0.5,  0.5,    -1.0,  0.0,  0.0,  // 10
  -0.5, -0.5,  0.5,    -1.0,  0.0,  0.0,  // 11

   0.5, -0.5, -0.5,     1.0,  0.0,  0.0,  // 12  Right
   0.5, -0.5,  0.5,     1.0,  0.0,  0.0,  // 13
   0.5,  0.5,  0.5,     1.0,  0.0,  0.0,  // 14
   0.5,  0.5, -0.5,     1.0,  0.0,  0.0,  // 15

   0.5,  0.5,  0.5,     0.0,  1.0,  0.0,  // 16  Top
  -0.5,  0.5,  0.5,     0.0,  1.0,  0.0,  // 17
  -0.5,  0.5, -0.5,     0.0,  1.0,  0.0,  // 18
   0.5,  0.5, -0.5,     0.0,  1.0,  0.0,  // 19

   0.5, -0.5,  0.5,     0.0, -1.0,  0.0,  // 20  Bottom
   0.5, -0.5, -0.5,     0.0, -1.0,  0.0,  // 21
  -0.5, -0.5, -0.5,     0.0, -1.0,  0.0,  // 22
  -0.5, -0.5,  0.5,     0.0, -1.0,  0.0,  // 23
];

pub const INDICES: [u32; 3 * 2 * 6] = [
   0,  2,  1,   0,  3,  2,  // Back
   4,  6,  5,   4,  7,  6,  // Front
   8, 10,  9,   8, 11, 10,  // Left
  12, 14, 13,  12, 15, 14,  // Right
  16, 18, 17,  16, 19, 18,  // Top
  20, 22, 21,  20, 23, 22,  // Bottom
];

pub const STRIDE: u32 = (3+3) * SIZE_F32;
pub const SKIPS: [u32; 2] = [0, 3 * SIZE_F32];

#[allow(dead_code)]
impl Cube {
  pub fn new(name: String) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    for i in 0..SKIPS.len() {
      vao.add_attribute(i as u32, STRIDE, SKIPS[i]);
    }

    vbo.send_data(&VERTICES);
    ebo.send_data(&INDICES);

    return Self {
      id: Uuid::new_v4(),
      name: name,
      transform: Transform::default(),
      material: Material::default(),
      vao,
      vbo,
      ebo,
    };
  }

  fn get_points_list(&self) -> Vec<Vec3> {
    return vec![
      Vec3::new(-0.5, -0.5, -0.5),
      Vec3::new( 0.5, -0.5, -0.5),
      Vec3::new( 0.5,  0.5, -0.5),
      Vec3::new(-0.5,  0.5, -0.5),
      Vec3::new(-0.5, -0.5,  0.5),
      Vec3::new( 0.5, -0.5,  0.5),
      Vec3::new( 0.5,  0.5,  0.5),
      Vec3::new(-0.5,  0.5,  0.5),
    ];
  }
}

impl Object for Cube {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Cube }

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
      gl::DrawElements(gl::TRIANGLES, INDICES.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
    }
  }

  fn clone(&self) -> Self {
    let mut cube = Cube::new(self.name.clone());
    cube.transform = self.transform.clone();
    cube.material = self.material.clone();
    return cube;
  }

  fn ray_intersection(&self, _ray: Ray) -> Option<f32> {
    unimplemented!()
  }

  fn can_generate_points_cloud(&self) -> bool { true }

  fn generate_points_cloud(&self) -> Option<PointsCloud> {
    let points = self.get_points_list();
    return Some(PointsCloud::new(format!("{}_points", self.name), points, vec![]));
  }

  fn generate_points_cloud_with_inner_samples(&self, inner_samples: u32) -> Option<PointsCloud> {
    let points = self.get_points_list();
    let mut inner_points = vec![];

    for _ in 0..inner_samples {
      inner_points.push(Vec3::new(rand::random(), rand::random(), rand::random()) - Vec3::ONE * 0.5);
    }

    return Some(PointsCloud::new(format!("{}_points", self.name), points, inner_points));
  }
}

implement_transformable!(Cube);
