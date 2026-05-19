use std::{any::Any, cell::RefCell, rc::Rc};

use glam::Vec3;

use crate::{opengl::program::Program, utils::transform::Transformable};

#[allow(dead_code)]
pub trait Light: Transformable {
  fn get_id(&self) -> uuid::Uuid;
  fn get_name(&self) -> String;
  fn get_name_mut(&mut self) -> &mut String;
  fn set_name(&mut self, name: String);

  fn get_position(&self) -> Vec3;
  fn get_position_mut(&mut self) -> &mut Vec3;

  fn get_diffuse(&self) -> Vec3;
  fn get_diffuse_mut(&mut self) -> &mut Vec3;
  fn set_diffuse(&mut self, diffuse: Vec3);

  fn get_ambient(&self) -> Vec3;
  fn get_ambient_mut(&mut self) -> &mut Vec3;
  fn set_ambient(&mut self, ambient: Vec3);

  fn get_specular(&self) -> Vec3;
  fn get_specular_mut(&mut self) -> &mut Vec3;
  fn set_specular(&mut self, specular: Vec3);

  fn tick(&mut self);
  fn draw(&self, program: &Program);

  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;

  fn clone(&self) -> Self where Self: Sized;
  fn clone_rc_ref(&self) -> Rc<RefCell<dyn Light>>;

  fn send_to_program(&self, program: &Program, index: usize) {
    program.set_uniform_vec3f(format!("lights[{}].position", index).as_str(), self.get_position()).expect("Failed to set light position");
    program.set_uniform_vec3f(format!("lights[{}].diffuse", index).as_str(), self.get_diffuse()).expect("Failed to set light diffuse");
    program.set_uniform_vec3f(format!("lights[{}].specular", index).as_str(), self.get_specular()).expect("Failed to set light specular");
    program.set_uniform_vec3f(format!("lights[{}].ambient", index).as_str(), self.get_ambient()).expect("Failed to set light ambient");
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LightType {
  PointLight,
}

#[macro_export]
macro_rules! implement_partial_Light {
  () => {
    fn get_id(&self) -> uuid::Uuid { self.id }
    fn get_name(&self) -> String { self.name.clone() }
    fn get_name_mut(&mut self) -> &mut String { &mut self.name }
    fn set_name(&mut self, name: String) { self.name = name }

    fn get_position(&self) -> Vec3 { self.transform.translation }
    fn get_position_mut(&mut self) -> &mut Vec3 { &mut self.transform.translation }

    fn get_diffuse(&self) -> Vec3 { self.diffuse }
    fn get_diffuse_mut(&mut self) -> &mut Vec3 { &mut self.diffuse }
    fn set_diffuse(&mut self, diffuse: Vec3) { self.diffuse = diffuse }

    fn get_ambient(&self) -> Vec3 { self.ambient }
    fn get_ambient_mut(&mut self) -> &mut Vec3 { &mut self.ambient }
    fn set_ambient(&mut self, ambient: Vec3) { self.ambient = ambient }

    fn get_specular(&self) -> Vec3 { self.specular }
    fn get_specular_mut(&mut self) -> &mut Vec3 { &mut self.specular }
    fn set_specular(&mut self, specular: Vec3) { self.specular = specular }

    fn as_any(&self) -> &dyn std::any::Any where Self: Sized { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any where Self: Sized { self }

    fn clone_rc_ref(&self) -> std::rc::Rc<std::cell::RefCell<dyn Light>> { std::rc::Rc::new(std::cell::RefCell::new(self.clone())) }
  };
}
