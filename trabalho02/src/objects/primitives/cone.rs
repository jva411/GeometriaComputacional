use core::f32;
use std::collections::HashSet;

use glam::Vec3;
use uuid::Uuid;


use crate::{geometry::octree::OctreeNode, implement_partial_Object, implement_transformable, objects::{geometry::convex_hull::ConvexHull, mesh::mesh::Mesh, object::{ConvexHullProps, Object, ObjectType}}, opengl::{ebo::EBO, renderer::Renderer, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform, vector::wed_points}};

pub struct Cone {
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

impl Cone {
  pub fn new(name: String, radius: f32, height: f32, subdivisions: u32) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let base_normal = Vec3::NEG_Y;
    let base_origin = Vec3::new(0.0, -height / 2.0, 0.0);

    vertices.push(base_origin);
    vertices.push(base_normal);

    let base_center_index = 0;

    for i in 0..=subdivisions {
      let angle = (i as f32 / subdivisions as f32) * f32::consts::PI * 2.0;
      let x = radius * angle.cos();
      let z = radius * angle.sin();

      vertices.push(Vec3::new(x, -height / 2.0, z));
      vertices.push(base_normal);

      if i > 0 {
        let current = base_center_index + i as usize + 1;
        let prev = base_center_index + i as usize;

        indices.push(base_center_index);
        indices.push(prev);
        indices.push(current);
      }
    }

    let side_start_index = vertices.len() / 2;

    for i in 0..=subdivisions {
      let angle = (i as f32 / subdivisions as f32) * f32::consts::PI * 2.0;
      let x = radius * angle.cos();
      let z = radius * angle.sin();

      let normal_x = height * angle.cos();
      let normal_y = radius;
      let normal_z = height * angle.sin();
      let side_normal = Vec3::new(normal_x, normal_y, normal_z).normalize();

      vertices.push(Vec3::new(0.0, height / 2.0, 0.0));
      vertices.push(side_normal);

      vertices.push(Vec3::new(x, -height / 2.0, z));
      vertices.push(side_normal);

      if i > 0 {
        let top_curr = side_start_index + (i as usize * 2);
        let bottom_curr = side_start_index + (i as usize * 2) + 1;
        let bottom_prev = side_start_index + ((i - 1) as usize * 2) + 1;

        indices.push(bottom_prev);
        indices.push(top_curr);
        indices.push(bottom_curr);
      }
    }

    vao.add_attribute(0, 6 * SIZE_F32, 0);
    vao.add_attribute(1, 6 * SIZE_F32, 3 * SIZE_F32);

    let flat_data: Vec<f32> = vertices.iter().flat_map(|v| v.to_array()).collect();
    vbo.send_data(&flat_data);
    ebo.send_data(&indices.iter().map(|i| *i as u32).collect::<Vec<u32>>());

    Cone {
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

  fn get_vertices(&self) -> (Vec<Vec3>, Vec<u32>) {
    let mut points = Vec::with_capacity(self.vertices.len() / 2);
    for i in 0..self.vertices.len() / 2 {
      points.push(self.vertices[i * 2]);
    }

    let faces = self.indices.iter().map(|&v| v as u32).collect::<Vec<_>>();

    return (points, faces);
  }
}

impl Object for Cone {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Cone }

  fn tick(&mut self) { }

  fn draw(&self, renderer: &mut Renderer, base_transform: Option<Transform>) {
    let program = &renderer.current_program;

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

  fn ray_intersection(&self, _ray: Ray) -> Option<f32> {
    unimplemented!()
  }

  fn contains_point(&self, point: Vec3) -> bool {
    let half_height = self.height / 2.0;
    if point.y > half_height || point.y < -half_height {
      return false;
    }

    let radius_at_y = self.radius * (self.height - (point.y + 0.5)) / self.height;
    return point.x * point.x + point.z * point.z <= radius_at_y * radius_at_y;
  }

  fn can_generate_convex_hull(&self) -> bool { true }

  fn convex_hull(&self, _use_parry: bool) -> Option<crate::objects::geometry::convex_hull::ConvexHull> {
    let (points, faces) = self.get_vertices();
    let mut hull_points = HashSet::new();
    for i in 0..points.len() {
      hull_points.insert(i);
    }

    let new_name = format!("{}_hull", self.name);
    let mesh = Mesh::new(new_name.clone(), points, vec![], faces);
    let mut hull = ConvexHull::new(new_name, mesh, hull_points);
    hull.transform = self.transform.clone();
    hull.material = self.material.clone();
    return Some(hull);
  }

  fn convex_hull_with_inner_samples(&self, props: ConvexHullProps) -> Option<ConvexHull> {
    let (mut points, faces) = self.get_vertices();
    let mut hull_points = HashSet::new();
    for i in 0..points.len() {
      hull_points.insert(i);
    }

    let mut inner_points = Vec::new();
    match props {
      ConvexHullProps::RandomPoints(inner_samples) => {
        for _ in 0..inner_samples {
          let mut point = (Vec3::new(rand::random(), rand::random(), rand::random()) * 2.0 - Vec3::ONE)
            * Vec3::new(self.radius, self.height, self.radius);
          while !self.contains_point(point) {
            point = (Vec3::new(rand::random(), rand::random(), rand::random()) * 2.0 - Vec3::ONE)
              * Vec3::new(self.radius, self.height, self.radius);
          }
          inner_points.push(point.as_dvec3());
        }
      }
      ConvexHullProps::OctreePoints => {
        let vertices_f64 = points.iter().map(Vec3::as_dvec3).collect::<Vec<_>>();
        let octree = OctreeNode::generate_from_mesh(&vertices_f64, &faces, 4);
        let octree_new_points = octree.get_leaves_centroids(&vertices_f64, &faces);
        inner_points = octree_new_points
      }
    }

    let inner_points = wed_points(&points, &inner_points, &faces);
    points.extend(inner_points);
    let new_name = format!("{}_hull", self.name);
    let mesh = Mesh::new(new_name.clone(), points, vec![], faces);
    let mut hull = ConvexHull::new(new_name, mesh, hull_points);
    hull.transform = self.transform.clone();
    hull.material = self.material.clone();
    return Some(hull);
  }
}

implement_transformable!(Cone);
