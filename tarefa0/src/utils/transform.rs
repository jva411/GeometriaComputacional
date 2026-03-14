use glam::{Mat4, Quat, Vec3};
use serde::Serialize;

use crate::opengl::program::Program;


#[derive(Debug, Clone, Serialize)]
pub struct Rotation {
  pub yaw: f32,
  pub pitch: f32,
  pub roll: f32,
}

impl Default for Rotation {
  fn default() -> Self {
    Rotation::new(0.0, 0.0, 0.0)
  }
}

impl Rotation {
  pub fn new(yaw: f32, pitch: f32, roll: f32) -> Self {
    Rotation {
      yaw,
      pitch,
      roll,
    }
  }

  pub fn to_quat(&self) -> Quat {
    Quat::from_axis_angle(Vec3::Y, self.yaw.to_radians())
      * Quat::from_axis_angle(Vec3::X, self.pitch.to_radians())
      * Quat::from_axis_angle(Vec3::Z, self.roll.to_radians())
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct Transform {
  pub translation: Vec3,
  pub rotation: Rotation,
  pub scale: Vec3,
}

impl Default for Transform {
  fn default() -> Self {
    Transform {
      translation: Vec3::ZERO,
      rotation: Rotation::default(),
      scale: Vec3::ONE,
    }
  }
}

#[allow(dead_code)]
impl Transform {
  pub fn translate3f(&mut self, x: f32, y: f32, z: f32) {
    self.translation += Vec3::new(x, y, z);
  }
  pub fn translatev3f(&mut self, v: Vec3) {
    self.translation += Vec3::new(v.x, v.y, v.z);
  }

  pub fn scale3f(&mut self, x: f32, y: f32, z: f32) {
    self.scale *= Vec3::new(x, y, z);
  }
  pub fn scalev3f(&mut self, v: Vec3) {
    self.scale *= Vec3::new(v.x, v.y, v.z);
  }

  pub fn add_yaw(&mut self, radians: f32) {
    self.rotation.yaw += radians;
  }
  pub fn add_pitch(&mut self, radians: f32) {
    self.rotation.pitch += radians;
  }
  pub fn add_roll(&mut self, radians: f32) {
    self.rotation.roll += radians;
  }

  pub fn build_model(&self) -> Mat4 {
    Mat4::from_scale_rotation_translation(self.scale, self.rotation.to_quat(), self.translation)
  }

  pub fn concat(&self, other: &Transform) -> Transform {
    Transform {
      translation: self.translation + other.translation,
      rotation: Rotation {
        yaw: self.rotation.yaw + other.rotation.yaw,
        pitch: self.rotation.pitch + other.rotation.pitch,
        roll: self.rotation.roll + other.rotation.roll,
      },
      scale: self.scale * other.scale,
    }
  }

  pub fn send_to_program(&self, program: &Program) {
    program.set_uniform_matrix4f("model", self.build_model()).unwrap();
  }
}

pub trait Transformable {
  fn get_transform(&self) -> &Transform;
  fn get_transform_mut(&mut self) -> &mut Transform;
}

#[macro_export]
macro_rules! implement_transformable {
  ($type:ty) => {
    impl crate::utils::transform::Transformable for $type {
      fn get_transform(&self) -> &Transform { &self.transform }
      fn get_transform_mut(&mut self) -> &mut Transform { &mut self.transform }
    }
  };
}
