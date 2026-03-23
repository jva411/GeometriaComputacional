use std::path::PathBuf;

use glam::Vec3;
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::{geometry::points_cloud::PointsCloud, object::{Object, ObjectType}}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform, vector::calculate_normals}};

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
  pub fn new(name: String, vertices: Vec<Vec3>, normals: Vec<Vec3>, faces: Vec<u32>) -> Self {
    let normals = if normals.is_empty() {
      calculate_normals(&vertices, &faces)
    } else {
      normals
    };

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

    let stride = 6 * SIZE_F32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, 3 * SIZE_F32);

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

  pub fn from_obj_file(name: String, file_path: PathBuf, scale: f32) -> Self {
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

    if normals.is_empty() {
      normals = calculate_normals(&vertices, &faces);
    }

    let mut mesh = Mesh::new(name, vertices, normals, faces);
    mesh.transform.scale3f(scale, scale, scale);

    return mesh;
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

  fn clone(&self) -> Self {
    let mut mesh = Mesh::new(self.name.clone(), self.vertices.clone(), self.normals.clone(), self.faces.clone());
    mesh.transform = self.transform.clone();
    mesh.material = self.material.clone();
    return mesh;
  }

  fn ray_intersection(&self, _ray: Ray) -> Option<f32> {
    todo!()
  }

  fn can_generate_points_cloud(&self) -> bool { true }
  fn generate_points_cloud(&self) -> Option<PointsCloud> {
    let points = self.vertices.clone();
    let mut cloud = PointsCloud::new(format!("{}_points", self.name), points, vec![]);
    cloud.transform = self.transform.clone();
    return Some(cloud);
  }

  fn generate_points_cloud_with_inner_samples(&self, _inner_samples: u32) -> Option<PointsCloud> {
    return self.generate_points_cloud();
  }
}

implement_transformable!(Mesh);
