use std::{cell::RefCell, rc::Rc};

use egui::ahash::{HashMap, HashMapExt};
use glam::Vec3;
use uuid::Uuid;

use crate::{lights::light::Light, objects::{gizmo::gizmo::Gizmo, grid::{axes::Axes, grid::Grid}, object::Object}, opengl::{program::Program, renderer::{ProgramType, Renderer}}, utils::{camera::Camera, transform::Transformable}};

#[allow(dead_code)]
pub struct Scene {
  pub camera: Camera,
  pub objects: Vec<Rc<RefCell<dyn Object>>>,
  pub objects_by_id: HashMap<Uuid, Rc<RefCell<dyn Object>>>,
  pub objects_by_program: HashMap<ProgramType, Vec<Rc<RefCell<dyn Object>>>>,
  pub lights: Vec<Rc<RefCell<dyn Light>>>,
  pub lights_by_id: HashMap<Uuid, Rc<RefCell<dyn Light>>>,
  pub renderer: Rc<RefCell<Renderer>>,
  pub grid: Grid,
  pub axes: Axes,
  pub selected_object: SceneSelectedObject,
}

#[allow(dead_code)]
impl Scene {
  pub fn new(camera: Camera, renderer: Rc<RefCell<Renderer>>) -> Self {
    return Scene {
      camera,
      objects: Vec::new(),
      objects_by_id: HashMap::new(),
      objects_by_program: HashMap::new(),
      lights: Vec::new(),
      lights_by_id: HashMap::new(),
      renderer,
      grid: Grid::new(20, 0.5, Vec3::splat(0.5)),
      axes: Axes::new(),
      selected_object: SceneSelectedObject::None,
    };
  }

  pub fn tick(&mut self) {
    for object in &mut self.objects {
      object.borrow_mut().tick();
    }

    for light in &mut self.lights {
      light.borrow_mut().tick();
    }
  }

  pub fn draw(&self) {
    let mut renderer = self.renderer.borrow_mut();

    self.draw_objects(&mut renderer);
    self.draw_lights(&mut renderer);
    self.draw_grid(&mut renderer);
    self.draw_axes(&mut renderer);
  }

  fn draw_objects(&self, renderer: &mut Renderer) {
    for (program_type, objects) in &self.objects_by_program {
      renderer.bind_program(*program_type);
      let program = &renderer.current_program;

      self.camera.send_to_program(&program);
      let has_lights = program.set_uniform1i("n_lights", self.lights.len() as i32).is_ok();
      if has_lights {
        for (i, light) in self.lights.iter().enumerate() {
          light.borrow().send_to_program(&program, i);
        }
      }

      for object in objects {
        object.borrow().draw(program, None);
      }
    }
  }

  fn draw_lights(&self, renderer: &mut Renderer) {
    renderer.bind_program(ProgramType::Light);
    let program = &renderer.current_program;
    self.camera.send_to_program(&program);
    for light in &self.lights {
      light.borrow().draw(program);
    }
  }

  fn draw_grid(&self, renderer: &mut Renderer) {
    renderer.bind_program(ProgramType::Grid);
    let program = &renderer.current_program;
    self.camera.send_to_program(&program);
    self.grid.draw(program);
  }

  fn draw_axes(&self, renderer: &mut Renderer) {
    renderer.bind_program(ProgramType::Grid);
    let program = &renderer.current_program;
    unsafe { gl::Disable(gl::DEPTH_TEST); }

    self.axes.draw(program, None);
    self.selected_object.draw(program);

    unsafe { gl::Enable(gl::DEPTH_TEST); }
  }

  pub fn add_object(&mut self, object: Rc<RefCell<dyn Object>>) {
    let id = object.borrow().get_id();
    self.objects.push(object.clone());
    self.objects_by_id.insert(id, object.clone());
    self.objects_by_program.entry(object.borrow().get_program_type()).or_default().push(object.clone());
  }

  pub fn remove_object(&mut self, id: Uuid) {
    let removed_object = self.objects_by_id.remove(&id);
    if removed_object.is_none() {
      return
    };

    let removed_object = removed_object.unwrap();
    if let Some(objects) = self.objects_by_program.get_mut(&removed_object.borrow().get_program_type()) {
      if let Some(index) = objects.iter().position(|object| object.borrow().get_id() == id) {
        objects.remove(index);
      }
    }

    if let Some(index) = self.objects.iter().position(|object| object.borrow().get_id() == id) {
      self.objects.remove(index);
    }
  }

  pub fn add_light(&mut self, light: Rc<RefCell<dyn Light>>) {
    let id = light.borrow().get_id();
    self.lights.push(light.clone());
    self.lights_by_id.insert(id, light);
  }

  pub fn remove_light(&mut self, id: Uuid) {
    if self.lights_by_id.remove(&id).is_none() {
      return
    };

    if let Some(index) = self.lights.iter().position(|light| light.borrow().get_id() == id) {
      self.lights.remove(index);
    }
  }

  pub fn get_transformable_by_id(&self, id: Uuid) -> Option<Rc<RefCell<dyn Transformable>>> {
    let object = self.objects_by_id.get(&id);
    if object.is_some() {
      return Some(object.unwrap().clone());
    }

    let light = self.lights_by_id.get(&id);
    if light.is_some() {
      return Some(light.unwrap().clone());
    }

    return None;
  }

  pub fn screenshot(&self, width: u32, height: u32) -> Vec<u8> {
    return self.renderer.borrow().read_pixels(width, height);
  }
}

#[derive(Clone)]
pub enum SceneSelectedObject {
  Object{ id: Uuid, gizmo: Rc<RefCell<dyn Gizmo>> },
  Light{ id: Uuid, gizmo: Rc<RefCell<dyn Gizmo>> },
  None,
}

impl SceneSelectedObject {
  pub fn is_none(&self) -> bool {
    match self {
      SceneSelectedObject::None => true,
      _ => false
    }
  }

  pub fn get_fields(&self) -> (Uuid, Rc<RefCell<dyn Gizmo>>) {
    match self {
      SceneSelectedObject::Object{ id, gizmo } => (*id, gizmo.clone()),
      SceneSelectedObject::Light{ id, gizmo } => (*id, gizmo.clone()),
      SceneSelectedObject::None => panic!("Tried to get fields from None"),
    }
  }

  pub fn draw(&self, program: &Program) {
    match self {
      SceneSelectedObject::Object{ gizmo, .. } => gizmo.borrow().draw(program),
      SceneSelectedObject::Light{ gizmo, .. } => gizmo.borrow().draw(program),
      SceneSelectedObject::None => (),
    }
  }
}
