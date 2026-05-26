use std::collections::HashSet;

use glam::Vec3;
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::{mesh::mesh::Mesh, object::{Object, ObjectType}}, opengl::{program::Program, vao::VAO, vbo::VBO} , utils::{material::Material, transform::Transform}};

pub struct ConvexHull {
  pub id: Uuid,
  pub name: String,

  pub transform: Transform,
  pub material: Material,

  pub mesh: Mesh,
  pub hull_vertices: HashSet<usize>,

  pub render_points: bool,
  vao: VAO,
  vbo: VBO,
}

impl ConvexHull {
  pub fn new(name: String, mesh: Mesh, hull_vertices: HashSet<usize>) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    vao.bind();
    vbo.bind();

    let inner_points_color = Vec3::new(0.0, 1.0, 0.0);
    let hull_points_color = Vec3::new(1.0, 1.0, 1.0);
    let vertex_data = mesh.vertices.iter()
      .enumerate()
      .flat_map(|(i, point)| {
        let color = if hull_vertices.contains(&i) { hull_points_color } else { inner_points_color };
        vec![point.x, point.y, point.z, color.x, color.y, color.z]
      })
      .collect::<Vec<f32>>();

    vbo.send_data(&vertex_data);
    let stride = (6 * std::mem::size_of::<f32>()) as u32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, (3 * std::mem::size_of::<f32>()) as u32);

    let mesh = Self {
      id: Uuid::new_v4(),
      name,

      transform: Transform::default(),
      material: Material::default(),

      mesh,
      hull_vertices,

      render_points: true,
      vao,
      vbo,
    };

    return mesh;
  }
}

impl Object for ConvexHull {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::ConvexHull }

  fn tick(&mut self) {
    self.mesh.transform = self.transform.clone();
    self.mesh.material = self.material.clone();
  }

  fn draw(&self, program: &Program, base_transform: Option<Transform>) {
    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };
    self.mesh.draw(program, Some(model_transform.clone()));

    self.vao.bind();
    self.vbo.bind();
    model_transform.send_to_program(program);
    unsafe {
      if self.render_points {
        gl::DrawArrays(gl::POINTS, 0, self.mesh.vertices.len() as i32);
      }
    }
  }

  fn clone(&self) -> Self {
    let mut mesh = Self::new(self.name.clone(), self.mesh.clone(), self.hull_vertices.clone());
    mesh.transform = self.transform.clone();
    mesh.material = self.material.clone();
    return mesh;
  }
}

implement_transformable!(ConvexHull);
