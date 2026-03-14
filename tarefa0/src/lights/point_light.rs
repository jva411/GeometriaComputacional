use glam::Vec3;
use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::{implement_partial_Light, implement_transformable, lights::light::Light, objects::primitives::cube::{INDICES as CUBE_INDICES, SKIPS as CUBE_SKIPS, STRIDE as CUBE_STRIDE, VERTICES as CUBE_VERTICES}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::transform::Transform};

#[derive(Debug)]
pub struct PointLight {
  pub id: uuid::Uuid,
  pub name: String,

  pub transform: Transform,
  pub ambient: Vec3,
  pub diffuse: Vec3,
  pub specular: Vec3,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,
}

impl Serialize for PointLight {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut sate = serializer.serialize_struct("PointLight", 4)?;
    sate.serialize_field("position", &self.transform.translation)?;
    sate.serialize_field("ambient", &self.ambient)?;
    sate.serialize_field("diffuse", &self.diffuse)?;
    sate.serialize_field("specular", &self.specular)?;
    sate.end()
  }
}

impl PointLight {
  pub fn new(name: String, ambient: Vec3, diffuse: Vec3, specular: Vec3) -> PointLight {
    let id = uuid::Uuid::new_v4();

    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    for i in 0..CUBE_SKIPS.len() {
      vao.add_attribute(i as u32, CUBE_STRIDE, CUBE_SKIPS[i]);
    }

    vbo.send_data(&CUBE_VERTICES);
    ebo.send_data(&CUBE_INDICES);

    PointLight {
      id,
      name,

      transform: Transform::default(),
      ambient,
      diffuse,
      specular,

      vao,
      vbo,
      ebo,
    }
  }
}

#[allow(dead_code)]
impl Light for PointLight {
  implement_partial_Light!();

  fn clone(&self) -> Self {
    let mut clone = Self::new(self.name.clone(), self.ambient, self.diffuse, self.specular);
    clone.transform = self.transform.clone();
    return clone;
  }

  fn tick(&mut self) { }

  fn draw(&self, program: &Program) {
    self.vao.bind();
    self.vbo.bind();
    self.ebo.bind();

    self.transform.send_to_program(&program);
    program.set_uniform_vec3f("lightColor", self.diffuse).unwrap();

    unsafe {
      gl::DrawElements(gl::TRIANGLES, CUBE_INDICES.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
    }
  }
}

implement_transformable!(PointLight);
