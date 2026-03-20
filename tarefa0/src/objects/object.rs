use std::{any::Any, cell::RefCell, rc::Rc};

use glam::Vec3;
use uuid::Uuid;

use crate::{objects::geometry::points_cloud::PointsCloud, opengl::{program::Program, renderer::ProgramType}, utils::{material::Material, ray::Ray, transform::{Transform, Transformable}}};

#[allow(dead_code, unused_variables)]
pub trait Object: Transformable {
  fn get_id(&self) -> Uuid;
  fn get_name(&self) -> String;
  fn get_name_mut(&mut self) -> &mut String;
  fn set_name(&mut self, name: String);
  fn get_type(&self) -> ObjectType;
  fn get_program_type(&self) -> ProgramType { ProgramType::Common }

  fn get_material(&self) -> &Material;
  fn get_material_mut(&mut self) -> &mut Material;

  fn tick(&mut self);
  fn draw(&self, program: &Program, base_transform: Option<Transform>);

  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;

  fn clone(&self) -> Self where Self: Sized;
  fn clone_rc_ref(&self) -> Rc<RefCell<dyn Object>>;

  fn ray_intersection(&self, ray: Ray) -> Option<f32>;
  fn contains_point(&self, point: Vec3) -> bool { unimplemented!() }

  fn can_generate_points_cloud(&self) -> bool { false }
  fn generate_points_cloud(&self) -> Option<PointsCloud> { None }
  fn generate_points_cloud_with_inner_samples(&self, inner_samples: u32) -> Option<PointsCloud> { None }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ObjectType {
  Cube,
  Sphere,
  Cylinder,
  Cone,
  Mesh,
  Square,
  PointsCloud,
}

#[macro_export]
macro_rules! implement_partial_Object {
  () => {
    fn get_id(&self) -> Uuid { self.id }
    fn get_name(&self) -> String { self.name.clone() }
    fn get_name_mut(&mut self) -> &mut String { &mut self.name }
    fn set_name(&mut self, name: String) { self.name = name }

    fn get_material(&self) -> &Material { &self.material }
    fn get_material_mut(&mut self) -> &mut Material { &mut self.material }

    fn as_any(&self) -> &dyn std::any::Any where Self: Sized { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any where Self: Sized { self }

    fn clone_rc_ref(&self) -> std::rc::Rc<std::cell::RefCell<dyn Object>> { std::rc::Rc::new(std::cell::RefCell::new(self.clone())) }
  };
}
