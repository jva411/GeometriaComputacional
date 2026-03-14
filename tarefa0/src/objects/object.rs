use std::{any::Any, cell::RefCell, rc::Rc};

use uuid::Uuid;

use crate::{opengl::program::Program, utils::{material::Material, ray::Ray, transform::{Transform, Transformable}}};

#[allow(dead_code)]
pub trait Object: Transformable {
  fn get_id(&self) -> Uuid;
  fn get_name(&self) -> String;
  fn get_name_mut(&mut self) -> &mut String;
  fn set_name(&mut self, name: String);
  fn get_type(&self) -> ObjectType;

  fn get_material(&self) -> &Material;
  fn get_material_mut(&mut self) -> &mut Material;

  fn tick(&mut self);
  fn draw(&self, program: &Program, base_transform: Option<Transform>);

  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;

  fn clone(&self) -> Self where Self: Sized;
  fn clone_rc_ref(&self) -> Rc<RefCell<dyn Object>>;

  fn ray_intersection(&self, ray: Ray) -> Option<f32>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ObjectType {
  Cube,
  Sphere,
  Cylinder,
  Cone,
  Generic,
  Square,
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

    fn clone_rc_ref(&self) -> Rc<RefCell<dyn Object>> { Rc::new(RefCell::new(self.clone())) }
  };
}
