use glam::Vec3;
use crate::{opengl::{vao::VAO, vbo::VBO, program::Program}, utils::transform::Transform};

#[allow(dead_code)]
pub struct Grid {
  pub vao: VAO,
  pub vbo: VBO,
  pub transform: Transform,
  vertex_count: i32,
}

#[allow(dead_code)]
impl Grid {
  pub fn new(n_steps: i32, step: f32, color: Vec3) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    vao.bind();
    vbo.bind();

    let size = n_steps as f32 * step;
    let mut vertices: Vec<f32> = Vec::new();
    for i in -n_steps..=n_steps {
      let pos = i as f32 * step;

      vertices.extend_from_slice(&[-size, 0.0, pos]);
      vertices.extend_from_slice(&[color.x, color.y, color.z]);
      vertices.extend_from_slice(&[size, 0.0, pos]);
      vertices.extend_from_slice(&[color.x, color.y, color.z]);

      vertices.extend_from_slice(&[pos, 0.0, size]);
      vertices.extend_from_slice(&[color.x, color.y, color.z]);
      vertices.extend_from_slice(&[pos, 0.0, -size]);
      vertices.extend_from_slice(&[color.x, color.y, color.z]);
    }

    let vertex_count = (vertices.len() / 6) as i32;
    vbo.send_data(&vertices);

    let stride = (6 * std::mem::size_of::<f32>()) as u32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, (3 * std::mem::size_of::<f32>()) as u32);

    Self {
      vao,
      vbo,
      transform: Transform::default(),
      vertex_count,
    }
  }

  pub fn draw(&self, program: &Program) {
    self.vao.bind();
    self.vbo.bind();
    self.transform.send_to_program(program);

    unsafe {
      gl::DrawArrays(gl::LINES, 0, self.vertex_count);
    }
  }
}
