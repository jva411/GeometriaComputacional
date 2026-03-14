use glam::{Vec3, Vec4Swizzles};

use crate::utils::transform::Transform;


#[derive(Debug, Clone, Copy)]
pub struct Ray {
  pub origin: Vec3,
  pub direction: Vec3,
}

impl Ray {
  pub fn new(origin: Vec3, direction: Vec3) -> Self {
    return Ray { origin, direction };
  }

  pub fn transform_inverse(&self, transform: &Transform) -> Self {
    let model_inverse = transform.build_model().inverse();

    return Ray::new(
      (model_inverse * self.origin.extend(1.0)).xyz(),
      (model_inverse * self.direction.extend(0.0)).xyz(),
    );
  }

  pub fn hit_point(&self, t: f32) -> Vec3 {
    return self.origin + t * self.direction;
  }
}
