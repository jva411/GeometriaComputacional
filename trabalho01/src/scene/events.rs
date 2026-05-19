use sdl2::{event::Event, keyboard::{Keycode, Scancode}, mouse::MouseButton};

use crate::scene::{ui::ui::UICommand, window::Window};

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

pub struct EventsManager {
  pub camera_speed: f32,
  pub is_scene_focused: bool,
}

impl EventsManager {
  pub fn new() -> Self {
    return EventsManager {
      camera_speed: 2.5,
      is_scene_focused: false,
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

  fn on_mouse_button_down(&mut self, _mouse_btn: MouseButton, _x: i32, _y: i32) -> EventResult {
    if !self.can_interact_with_scene() {
      return EventResult::None;
    }

    return EventResult::None;
  }

  fn on_mouse_button_up(&mut self, _mouse_btn: MouseButton, _x: i32, _y: i32) -> EventResult {
    return EventResult::None;
  }

  fn on_mouse_motion(&mut self, xrel: i32, yrel: i32) -> EventResult {
    if !self.can_interact_with_scene() {
      return EventResult::None;
    }

    const CAMERA_SENSITIVITY: f32 = 0.5;

    let mouse_state = self.sdl.event_pump.mouse_state();
    if mouse_state.middle() || mouse_state.right() {
      self.scene.camera.transform.add_yaw((-xrel as f32 * CAMERA_SENSITIVITY).to_radians());
      self.scene.camera.transform.add_pitch((-yrel as f32 * CAMERA_SENSITIVITY).to_radians());
    }

    return EventResult::None;
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
    let mouse_state = self.sdl.event_pump.mouse_state();
    let x = mouse_state.x() as u32;

    if x >= self.canvas_width {
      return false;
    }

    if self.ui_manager.is_add_object_window_open && self.egui.context.wants_pointer_input() {
      return false;
    }

    if mouse_state.middle() || mouse_state.right() {
      return true;
    }

    if self.egui.context.is_using_pointer() {
      return false;
    }

    if self.egui.context.wants_keyboard_input() {
      return false;
    }

    return true;
  }
}
