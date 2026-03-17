use std::{cell::RefCell, rc::Rc};

use glam::Vec3;
use parry3d::{math::Vec3 as PVec3, transformation::convex_hull};
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::object::{Object, ObjectType}, opengl::{program::Program, renderer::ProgramType, vao::VAO, vbo::VBO}, utils::{material::Material, ray::Ray, transform::Transform}};

pub struct PointsCloud {
  pub id: Uuid,
  pub name: String,

  pub points: Vec<Vec3>,

  pub transform: Transform,
  pub material: Material,

  pub vao: VAO,
  pub vbo: VBO,
}

impl PointsCloud {
  pub fn new(name: String, points: Vec<Vec3>) -> Self {
    let mut material = Material::default();
    material.diffuse = Vec3::new(0.0, 1.0, 0.0);
    let vao = VAO::new();
    let vbo = VBO::new();
    vao.bind();
    vbo.bind();

    let color = material.diffuse;
    let vertex_data = points.iter()
      .flat_map(|point| vec![point.x, point.y, point.z, color.x, color.y, color.z])
      .collect::<Vec<f32>>();

    vbo.send_data(&vertex_data);
    let stride = (6 * std::mem::size_of::<f32>()) as u32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, (3 * std::mem::size_of::<f32>()) as u32);

    return Self {
      id: Uuid::new_v4(),
      name: name,
      points: points,
      transform: Transform::default(),
      material,
      vao,
      vbo,
    }
  }

  fn update_opengl(&mut self) {
    self.vbo.bind();
    let color = self.material.diffuse;
    let vertex_data = self.points.iter()
      .flat_map(|point| vec![point.x, point.y, point.z, color.x, color.y, color.z])
      .collect::<Vec<f32>>();

    self.vbo.update_data(0, &vertex_data);
  }

  pub fn convex_hull(&self) -> Self {
    let (points, indices) =  convex_hull(self.points
      .iter()
      .map(|glam_vec3| PVec3::new(glam_vec3.x, glam_vec3.y, glam_vec3.z))
      .collect::<Vec<PVec3>>()
      .as_slice()
    );

    let points = points
      .iter()
      .map(|pvec3| Vec3::new(pvec3.x, pvec3.y, pvec3.z))
      .collect::<Vec<Vec3>>();

    let mut cloud = PointsCloud::new(format!("{}_convex_hull", self.name).to_string(), points);
    cloud.material.diffuse = Vec3::new(1.0, 0.0, 0.0);
    cloud.update_opengl();
    return cloud;
  }
}

impl Object for PointsCloud {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::PointsCloud }
  fn get_program_type(&self) -> ProgramType { ProgramType::Grid }
  fn tick(&mut self) { }

  fn draw(&self, program: &Program, base_transform: Option<Transform>) {
    self.vao.bind();
    self.vbo.bind();

    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };
    model_transform.send_to_program(&program);

    unsafe {
      gl::DrawArrays(gl::POINTS, 0, self.points.len() as i32);
    }
  }

  fn clone(&self) -> Self {
    let mut points_cloud = PointsCloud::new(self.name.clone(), self.points.clone());
    points_cloud.transform = self.transform.clone();
    points_cloud.material = self.material.clone();
    return points_cloud;
  }

  fn ray_intersection(&self, _ray: Ray) -> Option<f32> {
    unimplemented!()
  }
}

implement_transformable!(PointsCloud);
