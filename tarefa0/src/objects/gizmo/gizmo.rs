use std::{cell::RefCell, rc::Rc};

use glam::Vec3;

use crate::{opengl::program::Program, utils::{ray::Ray, transform::Transformable}};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum GizmoType {
  Translate,
  Rotate,
  Scale,
}

#[allow(dead_code)]
pub trait Gizmo {
  fn get_gizmo_type(&self) -> GizmoType;
  fn get_transformable(&self) -> Rc<RefCell<dyn Transformable>>;

  fn draw(&self, program: &Program);

  fn ray_intersection(&self, ray: Ray) -> Option<Vec3>;
}

#[macro_export]
macro_rules! implement_partial_gizmo {
  () => {
    fn get_gizmo_type(&self) -> GizmoType { self.gizmo_type }
    fn get_transformable(&self) -> Rc<RefCell<dyn Transformable>> { self.transformable.clone() }
  };
}
