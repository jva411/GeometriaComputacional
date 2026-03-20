use std::path::PathBuf;

use glam::Vec3;
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::object::{Object, ObjectType}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::{material::Material, ray::Ray, transform::Transform}};

pub struct Mesh {
  pub id: Uuid,
  pub name: String,

  pub transform: Transform,
  pub material: Material,

  pub vertices: Vec<Vec3>,
  pub normals: Vec<Vec3>,
  pub faces: Vec<u32>,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,
}

impl Mesh {
  pub fn new(name: String, file_path: PathBuf) -> Self {
    let (models, _) = tobj::load_obj(
      file_path,
      &tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
      },
    )
    .expect("Failed to load obj file");

    let mut vertices: Vec<Vec3> = Vec::new();
    let mut normals = Vec::new();
    let mut faces: Vec<u32> = Vec::new();

    for model in models {
      let mesh = &model.mesh;
      let vertex_offset = vertices.len() as u32;

      vertices.extend(
        mesh
          .positions
          .chunks_exact(3)
          .map(|p| Vec3::new(p[0], p[1], p[2])),
      );

      normals.extend(
        mesh
          .normals
          .chunks_exact(3)
          .map(|p| Vec3::new(p[0], p[1], p[2])),
      );

      faces.extend(mesh.indices.iter().map(|i| *i as u32 + vertex_offset));
    }

    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();

    vao.bind();
    vbo.bind();
    ebo.bind();
    vbo.send_data(
      &vertices
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| [v.x, v.y, v.z, n.x, n.y, n.z])
        .flatten()
        .collect::<Vec<f32>>(),
    );
    ebo.send_data(&faces);

    vao.add_attribute(0, 3, 0);
    vao.add_attribute(1, 3, 3);

    Self {
      id: Uuid::new_v4(),
      name,

      transform: Transform::default(),
      material: Material::default(),

      vertices,
      normals,
      faces,

      vao,
      vbo,
      ebo,
    }
  }
}

impl Object for Mesh {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Mesh }

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
      gl::DrawElements(gl::TRIANGLES, self.faces.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
    }
  }

  fn clone(&self) -> Self where Self: Sized {
    todo!()
  }

  fn ray_intersection(&self, ray: Ray) -> Option<f32> {
    todo!()
  }
}

implement_transformable!(Mesh);
