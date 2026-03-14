use std::{cell::RefCell, rc::Rc};

use egui::{vec2, ComboBox, FullOutput, SidePanel, Ui, Window as EguiWindow};
use uuid::Uuid;

use crate::{objects::gizmo::translate::Translate, scene::{scene::{Scene, SceneSelectedObject}, ui::{light_ui::NewLightProperties, object_ui::NewObjectProperties}, window::Window}};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UITab {
  Objects,
  Properties,
}

#[derive(Clone)]
pub enum UICommand {
  CreateObject(CreatingObject),
  DeleteObject(SelectedObject),
  CloneObject(SelectedObject),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedObject {
  Object(Uuid),
  Light(Uuid),
  None,
}

impl SelectedObject {
  pub fn to_scene(&self, scene: &Scene) -> SceneSelectedObject {
    match self {
      SelectedObject::Object(id) => SceneSelectedObject::Object {
        id: *id,
        gizmo: Rc::new(RefCell::new(Translate::new(scene.objects_by_id.get(id).unwrap().clone()))),
      },
      SelectedObject::Light(id) => SceneSelectedObject::Light {
        id: *id,
        gizmo: Rc::new(RefCell::new(Translate::new(scene.lights_by_id.get(id).unwrap().clone()))),
      },
      SelectedObject::None => SceneSelectedObject::None,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreatingObjectType {
  Object,
  Light,
}

impl Default for CreatingObjectType {
  fn default() -> Self {
    CreatingObjectType::Object
  }
}

#[derive(Clone, Debug)]
pub enum CreatingObject {
  Object(NewObjectProperties),
  Light(NewLightProperties),
}

impl Default for CreatingObject {
  fn default() -> Self {
    CreatingObject::Object(NewObjectProperties::default())
  }
}

#[allow(dead_code)]
pub struct UIManager {
  pub selected_tab: UITab,
  pub selected_object: SelectedObject,

  pub is_add_object_window_open: bool,
  pub creating_object: CreatingObject,
  pub creating_object_type: CreatingObjectType,
  pub previous_creating_object_type: CreatingObjectType,

  pub commands_queue: Vec<UICommand>,
}

impl UIManager {
  pub fn new() -> Self {
    UIManager {
      selected_tab: UITab::Objects,
      selected_object: SelectedObject::None,
      is_add_object_window_open: false,
      creating_object: CreatingObject::default(),
      creating_object_type: CreatingObjectType::default(),
      previous_creating_object_type: CreatingObjectType::default(),
      commands_queue: Vec::new(),
    }
  }
}

impl Window {
  pub fn draw_ui(&mut self) {
    unsafe {
      gl::Disable(gl::DEPTH_TEST);
      gl::Disable(gl::CULL_FACE);
    }

    self.egui.state.input.time = Some(self.elapsed_time as f64);
    self.egui.context.begin_pass(self.egui.state.input.take());

    let ui_manager = &mut self.ui_manager;
    let scene = &mut self.scene;

    SidePanel::right("main_side_panel")
      .resizable(false)
      .exact_width((self.width - self.canvas_width) as f32)
      .show(&self.egui.context, |ui| {
          Window::draw_tabs(ui, ui_manager, scene);
      });

    Window::draw_add_object_window(&self.egui.context, ui_manager);

    let FullOutput {
      platform_output,
      textures_delta,
      shapes,
      pixels_per_point,
      ..
    } = self.egui.context.end_pass();

    self.egui
      .state
      .process_output(&self.sdl.window, &platform_output);

    let paint_jobs = self.egui.context.tessellate(shapes, pixels_per_point);
    self.egui.painter.paint_jobs(None, textures_delta, paint_jobs);
  }

  fn draw_tabs(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene) {
    ui.horizontal(|ui| {
      ui.selectable_value(&mut ui_manager.selected_tab, UITab::Objects, "Objects");
      ui.selectable_value(
        &mut ui_manager.selected_tab,
        UITab::Properties,
        "Properties",
      );
    });
    ui.separator();

    match ui_manager.selected_tab {
      UITab::Objects => Window::draw_objects_lists_tab(ui, ui_manager, scene),
      UITab::Properties => Window::draw_properties_tab(ui, ui_manager, scene),
    }
  }

  fn draw_objects_lists_tab(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene) {
    Window::draw_objects_list(ui, ui_manager, scene);

    ui.separator();
    Window::draw_lights_list(ui, ui_manager, scene);
  }

  fn draw_properties_tab(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene) {
    match ui_manager.selected_object {
      SelectedObject::Object(id) => Window::draw_object_properties(ui, ui_manager, scene, id),
      SelectedObject::Light(id) => Window::draw_light_properties(ui, ui_manager, scene, id),
      SelectedObject::None => (),
    }
  }

  fn draw_add_object_window(ctx: &egui::Context, ui_manager: &mut UIManager) {
    if !ui_manager.is_add_object_window_open {
      return;
    }

    let mut is_still_open = ui_manager.is_add_object_window_open;
    EguiWindow::new("Add new object")
      .open(&mut is_still_open)
      .collapsible(false)
      .resizable(false)
      .default_size(vec2(300.0, 350.0))
      .show(ctx, |ui| {
        ui.heading("Type");
        ComboBox::from_label("Select the type")
          .selected_text(format!("{:?}", ui_manager.creating_object_type))
          .show_ui(ui, |ui| {
            ui.selectable_value(
              &mut ui_manager.creating_object_type,
              CreatingObjectType::Object,
              "Object",
            );
            ui.selectable_value(
              &mut ui_manager.creating_object_type,
              CreatingObjectType::Light,
              "Light",
            );
          });

        ui.separator();

        if ui_manager.creating_object_type != ui_manager.previous_creating_object_type {
          ui_manager.creating_object = match ui_manager.creating_object_type {
            CreatingObjectType::Object => CreatingObject::Object(NewObjectProperties::default()),
            CreatingObjectType::Light => CreatingObject::Light(NewLightProperties::default()),
          };
          ui_manager.previous_creating_object_type = ui_manager.creating_object_type.clone();
        }

        match ui_manager.creating_object_type {
          CreatingObjectType::Object => { ui_manager.draw_object_creation_options(ui); }
          CreatingObjectType::Light => { ui_manager.draw_light_creation_options(ui); }
        }
    });

    if ui_manager.is_add_object_window_open {
      ui_manager.is_add_object_window_open = is_still_open;
    }
  }

  pub fn process_ui_commands(&mut self) {
    let commands: Vec<UICommand> = self.ui_manager.commands_queue.drain(..).collect();

    for command in commands {
      match command {
        UICommand::CreateObject(props) => {
          let selected_object = match props {
            CreatingObject::Object(props) => {
              let new_object = Window::create_object(props);
              let new_object_id = new_object.borrow().get_id().clone();
              self.scene.add_object(new_object);
              SelectedObject::Object(new_object_id)
            }
            CreatingObject::Light(props) => {
              let new_light = Window::create_light(props);
              let new_light_id = new_light.borrow().get_id().clone();
              self.scene.add_light(new_light);
              SelectedObject::Light(new_light_id)
            }
          };
          self.ui_manager.selected_object = selected_object.clone();
          self.ui_manager.selected_tab = UITab::Properties;
          self.scene.selected_object = selected_object.to_scene(&self.scene);
        }

        UICommand::DeleteObject(props) => {
          match props {
            SelectedObject::Object(id) => { self.scene.remove_object(id); }
            SelectedObject::Light(id) => { self.scene.remove_light(id); }
            SelectedObject::None => {}
          }
        }

        UICommand::CloneObject(selected_object) => {
          match selected_object {
            SelectedObject::Object(id) => { self.clone_object(id); }
            SelectedObject::Light(id) => { self.clone_light(id); }
            SelectedObject::None => {}
          }
        }
      }
    }
  }

  pub fn select_object(&mut self, selected_object: SelectedObject) {
    self.ui_manager.selected_object = selected_object.clone();
    self.scene.selected_object = selected_object.to_scene(&self.scene);
  }
}
