use std::{cell::RefCell, rc::Rc};

use core::f32;

use glam::{Vec3, Vec3Swizzles};
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::{geometry::points_cloud::PointsCloud, object::{Object, ObjectType}}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform}};


pub struct Cylinder {
  pub id: Uuid,
  pub name: String,

  pub radius: f32,
  pub height: f32,
  pub subdivisions: u32,

  pub transform: Transform,
  pub material: Material,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,

  vertices: Vec<Vec3>,
  indices: Vec<usize>,
}

impl Cylinder {
  pub fn new(name: String, radius: f32, height: f32, subdivisions: u32) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let circles_normals = vec![
      Vec3::Y,
      Vec3::NEG_Y,
    ];
    let circles_origins = vec![
      Vec3::new(0.0, height / 2.0, 0.0),
      Vec3::new(0.0, -height / 2.0, 0.0),
    ];

    for (&normal, &origin) in circles_normals.iter().zip(circles_origins.iter()) {
      vertices.push(origin);
      vertices.push(normal);
      let start_index = vertices.len() / 2;
      for i in 0..=subdivisions {
        let angle = (i as f32 / subdivisions as f32) * f32::consts::PI * 2.0;
        let x = radius * angle.cos() + origin.x;
        let z = radius * angle.sin() + origin.z;

        vertices.push(Vec3::new(x, origin.y, z));
        vertices.push(normal);

        if i > 0 {
          if normal.y > 0.0 {
            indices.push(start_index - 1);
            indices.push(start_index + i as usize);
            indices.push(start_index + (i-1) as usize);
          } else {
            indices.push(start_index - 1);
            indices.push(start_index + (i-1) as usize);
            indices.push(start_index + i as usize);
          }
        }
      }

      if normal.y > 0.0 {
        indices.push(start_index - 1);
        indices.push(start_index + subdivisions as usize);
        indices.push(start_index);
      } else {
        indices.push(start_index - 1);
        indices.push(start_index);
        indices.push(start_index + subdivisions as usize);
      }
    }

    let start_index = vertices.len() / 2;
    for i in 0..=subdivisions {
      let angle = (i as f32 / subdivisions as f32) * f32::consts::PI * 2.0;
      let x = radius * angle.cos();
      let z = radius * angle.sin();
      let normal = Vec3::new(x, 0.0, z).normalize();

      vertices.push(Vec3::new(x, height / 2.0, z));
      vertices.push(normal);
      vertices.push(Vec3::new(x, -height / 2.0, z));
      vertices.push(normal);

      if i > 0 {
        let top_left = start_index + (i - 1) as usize * 2;
        let bottom_left = start_index + (i - 1) as usize * 2 + 1;
        let top_right = start_index + i as usize * 2;
        let bottom_right = start_index + i as usize * 2 + 1;

        indices.push(bottom_left);
        indices.push(top_left);
        indices.push(bottom_right);

        indices.push(top_left);
        indices.push(top_right);
        indices.push(bottom_right);
      }
    }

    let top_left = start_index + (subdivisions - 1) as usize;
    let bottom_left = start_index + (subdivisions - 1) as usize;
    let top_right = start_index + subdivisions as usize;
    let bottom_right = start_index + subdivisions as usize;

    indices.push(bottom_left);
    indices.push(bottom_right);
    indices.push(top_left);

    indices.push(top_left);
    indices.push(bottom_right);
    indices.push(top_right);

    vao.add_attribute(0, 6 * SIZE_F32, 0);
    vao.add_attribute(1, 6 * SIZE_F32, 3 * SIZE_F32);

    let flat_data: Vec<f32> = vertices.iter().flat_map(|v| v.to_array()).collect();
    vbo.send_data(&flat_data);
    ebo.send_data(&indices.iter().map(|i| *i as u32).collect::<Vec<u32>>());

    Cylinder {
      id: Uuid::new_v4(),
      name,
      radius,
      height,
      subdivisions,
      transform: Transform::default(),
      material: Material::default(),
      vao,
      vbo,
      ebo,
      vertices,
      indices,
    }
  }
}

impl Object for Cylinder {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Cylinder }

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
    let mut clone = Self::new(self.name.clone(), self.radius, self.height, self.subdivisions);
    clone.transform = self.transform.clone();
    clone.material = self.material.clone();
    return clone;
  }

  fn ray_intersection(&self, ray: Ray) -> Option<f32> {
    let half_height = self.height * 0.5;

    let co = ray.origin;
    let rd = ray.direction;

    let a = rd.xz().dot(rd.xz());
    let b = 2.0 * co.xz().dot(rd.xz());
    let c = co.xz().dot(co.xz()) - self.radius * self.radius;
    let delta = b * b - 4.0 * a * c;

    if delta < 0.0 {
      return None;
    }

    let delta_sqrt = delta.sqrt();
    let t1 = (-b - delta_sqrt) / (2.0 * a);
    let t2 = (-b + delta_sqrt) / (2.0 * a);

    let t = if t1 > 0.0 { t1 } else { t2 };

    let local_point = ray.origin + t * ray.direction;
    if local_point.y < -half_height || local_point.y > half_height {
      return None;
    }

    return Some(t);
  }

  fn contains_point(&self, point: Vec3) -> bool {
    let half_height = self.height * 0.5;
    return point.y >= -half_height
      && point.y <= half_height
      && point.x * point.x + point.z * point.z <= self.radius * self.radius;
  }

  fn can_generate_points_cloud(&self) -> bool { true }

  fn generate_points_cloud(&self) -> Option<PointsCloud> {
    let points = self.vertices.clone();
    return Some(PointsCloud::new(format!("{}_points", self.name), points, vec![]));
  }

  fn generate_points_cloud_with_inner_samples(&self, inner_samples: u32) -> Option<PointsCloud> {
    let points = self.vertices.clone();
    let mut inner_points = vec![];

    for _ in 0..inner_samples {
      let mut point = (Vec3::new(rand::random(), rand::random(), rand::random()) * 2.0 - Vec3::ONE)
        * Vec3::new(self.radius, self.height, self.radius);

      while !self.contains_point(point) {
        point = (Vec3::new(rand::random(), rand::random(), rand::random()) * 2.0 - Vec3::ONE)
          * Vec3::new(self.radius, self.height, self.radius);
      }
      inner_points.push(point);
    }

    return Some(PointsCloud::new(format!("{}_points", self.name), points, inner_points));
  }
}

implement_transformable!(Cylinder);
