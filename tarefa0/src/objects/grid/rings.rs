
use crate::{opengl::{ebo::EBO, vao::VAO, vbo::VBO}, utils::{transform::Transform}};

#[allow(dead_code)]
pub struct Rings {
  pub transform: Transform,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,
}

#[allow(dead_code)]
impl Rings {
  pub fn new() -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    // let subdivisions = 60;
    // let mut vertices = Vec::new();
    // let mut indices = Vec::new();

    return Self {
      transform: Transform::default(),
      vao,
      vbo,
      ebo,
    }
  }
}
