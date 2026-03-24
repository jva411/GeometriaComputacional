use std::{cell::RefCell, f32, rc::Rc};

use glam::{Vec2, Vec3};
use sdl2::{event::Event, keyboard::{Keycode, Scancode}, mouse::MouseButton};

use crate::{objects::{gizmo::gizmo::Gizmo, object::Object, primitives::planes::square::Square}, scene::{ui::ui::UICommand, window::Window}, utils::transform::Transformable};

pub enum EventResult {
  None,
  Quit,
}

macro_rules! match_event_result {
  ($expr:expr) => {
    match $expr {
      EventResult::Quit => return EventResult::Quit,
      _ => {}
    }
  };
}

#[allow(dead_code)]
pub struct PickingObject {
  pub object: Rc<RefCell<dyn Transformable>>,
  pub gizmo: Rc<RefCell<dyn Gizmo>>,
  pub axes: Vec3,
  pub square: Square,
}

impl PickingObject {
  pub fn new(object: Rc<RefCell<dyn Transformable>>, gizmo: Rc<RefCell<dyn Gizmo>>, axes: Vec3, square: Square) -> Self {
    return PickingObject { object, gizmo, axes, square };
  }
}

pub struct EventsManager {
  pub camera_speed: f32,
  pub is_scene_focused: bool,
  pub picking_object: Option<PickingObject>,
}

impl EventsManager {
  pub fn new() -> Self {
    return EventsManager {
      camera_speed: 2.5,
      is_scene_focused: false,
      picking_object: None
    };
  }
}

#[allow(dead_code)]
impl Window {
  pub fn proccess_events(&mut self) -> EventResult {
    let events: Vec<Event> = self.sdl.event_pump.poll_iter().collect();
    for event in events {
      match event {
        Event::Quit { .. } => return EventResult::Quit,
        Event::KeyUp { keycode: Some(key), .. } => match_event_result!(self.on_key_up(key)),
        Event::MouseWheel { y, .. } => match_event_result!(self.on_mouse_wheel(y)),
        Event::MouseButtonDown { mouse_btn, x, y, .. } => match_event_result!(self.on_mouse_button_down(mouse_btn, x, y)),
        Event::MouseButtonUp { mouse_btn, x, y, .. } => match_event_result!(self.on_mouse_button_up(mouse_btn, x, y)),
        Event::MouseMotion { xrel, yrel, .. } => match_event_result!(self.on_mouse_motion(xrel, yrel)),
        _ => {}
      }

      if !self.events_manager.is_scene_focused {
        self.egui.state.process_input(&self.sdl.window, event, &mut self.egui.painter);
      }
    }

    self.check_camera_movement();

    return EventResult::None;
  }

  fn on_key_up(&mut self, key: Keycode) -> EventResult {
    match key {
      Keycode::Escape => {
        if self.events_manager.is_scene_focused {
          self.events_manager.is_scene_focused = false;
          let mouse = self.sdl.context.mouse();
          self.sdl.context.mouse().set_relative_mouse_mode(false);
          mouse.warp_mouse_in_window(&self.sdl.window, self.width as i32/ 2, self.height as i32 / 2);
        }
      }
      Keycode::P => {
        self.ui_manager.commands_queue.push(UICommand::ScreenShot);
      }
      _ => {}
    };

    return EventResult::None;
  }

  fn on_mouse_wheel(&mut self, y: i32) -> EventResult {
    if !self.can_interact_with_scene() {
      return EventResult::None;
    }

    self.events_manager.camera_speed = (self.events_manager.camera_speed * (1.0 + y as f32 / 30.0)).clamp(0.2, 100.0);
    return EventResult::None;
  }

  fn on_mouse_button_down(&mut self, mouse_btn: MouseButton, x: i32, y: i32) -> EventResult {
    if !self.can_interact_with_scene() {
      return EventResult::None;
    }

    if mouse_btn == MouseButton::Left {
      self.check_mouse_picking(x as f32, y as f32);
    }

    return EventResult::None;
  }

  fn on_mouse_button_up(&mut self, mouse_btn: MouseButton, _x: i32, _y: i32) -> EventResult {
    if mouse_btn == MouseButton::Left {
      self.events_manager.picking_object = None;
    }
    if self.can_interact_with_scene() && mouse_btn == MouseButton::Left {
      // self.events_manager.is_scene_focused = true;
      // self.sdl.context.mouse().set_relative_mouse_mode(true);
    }

    return EventResult::None;
  }

  fn on_mouse_motion(&mut self, xrel: i32, yrel: i32) -> EventResult {
    if !self.can_interact_with_scene() {
      return EventResult::None;
    }

    const CAMERA_SENSITIVITY: f32 = 0.5;

    let mouse_state = self.sdl.event_pump.mouse_state();
    if self.events_manager.is_scene_focused || mouse_state.middle() {
      self.scene.camera.transform.add_yaw((-xrel as f32 * CAMERA_SENSITIVITY).to_radians());
      self.scene.camera.transform.add_pitch((-yrel as f32 * CAMERA_SENSITIVITY).to_radians());
    }

    if mouse_state.left() && self.events_manager.picking_object.is_some() {
      let picking_object = self.events_manager.picking_object.as_ref().unwrap();

      let object = picking_object.object.clone();
      let mut object = object.borrow_mut();

      let x1 = mouse_state.x() as f32;
      let y1 = mouse_state.y() as f32;

      let x0 = x1 - xrel as f32;
      let y0 = y1 - yrel as f32;

      let ray0 = self.scene.camera.get_ray(x0 / self.canvas_width as f32, y0 / self.height as f32);
      let hit_point0 = ray0.hit_point(picking_object.square.ray_intersection(ray0).unwrap());

      let ray1 = self.scene.camera.get_ray(x1 / self.canvas_width as f32, y1 / self.height as f32);
      let hit_point1 = ray1.hit_point(picking_object.square.ray_intersection(ray1).unwrap());

      let translation = (hit_point1 - hit_point0) * picking_object.axes;
      let object_transform = object.get_transform_mut();
      object_transform.translatev3f(translation);
    }

    return EventResult::None;
  }

  fn check_mouse_picking(&mut self, x0: f32, y0: f32) {
    if self.scene.selected_object.is_none() {
      return
    }

    let (id, gizmo) = self.scene.selected_object.get_fields();
    let transformable = self.scene.get_transformable_by_id(id);
    if transformable.is_none() {
      return
    }

    let ray0 = self.scene.camera.get_ray(x0 / self.canvas_width as f32, y0 / self.height as f32);
    let intersection = gizmo.borrow().ray_intersection(ray0);
    if intersection.is_none() {
      return
    }

    let object_rc = transformable.unwrap();
    let object = object_rc.borrow();
    let intersection = intersection.unwrap();

    let object_plane = Square::new(
      "".to_string(),
      object.get_transform().translation,
      self.scene.camera.transform.rotation * Vec3::Z,
      Vec2::new(f32::MAX, f32::MAX),
    );

    self.events_manager.picking_object = Some(PickingObject::new(
      object_rc.clone(),
      gizmo,
      intersection,
      object_plane,
    ));
  }

  fn check_camera_movement(&mut self) {
    if !self.can_interact_with_scene() {
      return;
    }

    let keyboard = self.sdl.event_pump.keyboard_state();
    let smoothness = self.events_manager.camera_speed * self.delta_time;

    let mut forward = 0.0;
    if keyboard.is_scancode_pressed(Scancode::W) {
      forward += 1.0;
    }
    if keyboard.is_scancode_pressed(Scancode::S) {
      forward += -1.0;
    }

    let mut right = 0.0;
    if keyboard.is_scancode_pressed(Scancode::D) {
      right += 1.0;
    }
    if keyboard.is_scancode_pressed(Scancode::A) {
      right += -1.0;
    }

    let mut up = 0.0;
    if keyboard.is_scancode_pressed(Scancode::Space) {
      up += 1.0;
    }
    if keyboard.is_scancode_pressed(Scancode::LShift) {
      up += -1.0;
    }

    self.scene.camera.transform.translate(forward * smoothness, right * smoothness, up * smoothness);
  }

  fn can_interact_with_scene(&self) -> bool {
    // let mouse_state = self.sdl.event_pump.mouse_state();
    // let x = mouse_state.x() as u32;

    return !(
      self.egui.context.is_using_pointer()
      || self.egui.context.wants_keyboard_input()
    );
  }
}
