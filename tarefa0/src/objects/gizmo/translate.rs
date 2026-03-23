use std::{cell::RefCell, rc::Rc};

use glam::Vec3;

use crate::{implement_partial_gizmo, objects::{gizmo::gizmo::{Gizmo, GizmoType}, grid::axes::Axes}, opengl::program::Program, utils::{ray::Ray, transform::Transformable}};

#[allow(dead_code)]
pub struct Translate {
  pub gizmo_type: GizmoType,
  pub transformable: Rc<RefCell<dyn Transformable>>,
  pub axes: Axes,
}

impl Translate {
  pub fn new(transformable: Rc<RefCell<dyn Transformable>>) -> Self {
    Self {
      gizmo_type: GizmoType::Translate,
      transformable,
      axes: Axes::new(),
    }
  }
}

impl Gizmo for Translate {
  implement_partial_gizmo!();

  fn draw(&self, program: &Program) {
    let mut transform = self.transformable.borrow().get_transform().clone();
    transform.scale = Vec3::ONE;
    self.axes.draw(program, Some(transform));
  }

  fn ray_intersection(&self, ray: Ray) -> Option<glam::Vec3> {
    let mut transform = self.transformable.borrow().get_transform().clone();
    transform.scale = Vec3::ONE;
    self.axes.ray_intersection(ray, &transform)
  }
}

impl Clone for Translate {
  fn clone(&self) -> Self {
    let clone = Self::new(self.transformable.clone());
    return clone;
  }
}
