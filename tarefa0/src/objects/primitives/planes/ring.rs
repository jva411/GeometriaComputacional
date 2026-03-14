use glam::Vec3;
use uuid::Uuid;

use crate::{opengl::{ebo::EBO, vao::VAO, vbo::VBO}, utils::{material::Material, transform::Transform}};

#[allow(dead_code)]
pub struct Ring {
  pub id: Uuid,
  pub name: String,

  pub origem: Vec3,
  pub normal: Vec3,
  pub inner_radius: f32,
  pub outer_radius: f32,

  pub transform: Transform,
  pub material: Material,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,
}
