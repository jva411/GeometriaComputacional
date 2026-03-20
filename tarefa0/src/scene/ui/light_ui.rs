use std::{cell::RefCell, rc::Rc};

use egui::{ComboBox, Ui};
use glam::Vec3;
use uuid::Uuid;

use crate::{lights::{light::{Light, LightType}, point_light::PointLight}, scene::{scene::{Scene, SceneSelectedObject}, ui::ui::{CreatingObject, CreatingObjectType, SelectedObject, UICommand, UIManager}, window::Window}};

#[derive(Clone, Debug)]
pub struct NewLightProperties {
  pub name: String,
  pub primitive: LightType,
  pub ambient: [f32; 3],
  pub diffuse: [f32; 3],
  pub specular: [f32; 3],
}

impl Default for NewLightProperties {
  fn default() -> Self {
    NewLightProperties {
      name: String::from("Light"),
      primitive: LightType::PointLight,
      ambient: [0.6, 0.6, 0.6],
      diffuse: [1.0, 1.0, 1.0],
      specular: [1.0, 1.0, 1.0],
    }
  }
}

impl Window {
  pub fn create_light(props: NewLightProperties) -> Rc<RefCell<dyn Light>> {
    match props.primitive {
      LightType::PointLight => {
        let light = PointLight::new(
          props.name,
          Vec3::from_array(props.ambient),
          Vec3::from_array(props.diffuse),
          Vec3::from_array(props.specular),
        );
        Rc::new(RefCell::new(light))
      }
    }
  }

  pub fn clone_light(&mut self, selected_id: Uuid) {
    let light = self.scene.lights_by_id.get(&selected_id);
    if light.is_none() {
      return
    }

    let light = light.unwrap().clone();
    let light = light.borrow();

    let new_light_rc = light.clone_rc_ref();
    let new_light_id = new_light_rc.borrow().get_id();
    {
      let mut new_light = new_light_rc.borrow_mut();
      new_light.set_name(format!("{} Copy", light.get_name()));
    }
    self.scene.add_light(new_light_rc);
    self.select_object(SelectedObject::Light(new_light_id));
  }

  pub fn draw_lights_list(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene) {
    if ui.button("Add Light").clicked() {
      ui_manager.is_add_object_window_open = true;
      ui_manager.creating_object_type = CreatingObjectType::Light;
      ui_manager.creating_object = CreatingObject::Light(NewLightProperties::default());
    }
    ui.separator();

    let mut sorted_lights = scene.lights_by_id
      .iter()
      .collect::<Vec<_>>();

    sorted_lights.sort_by_key(|(id, _)| scene.lights_by_id.get(id).unwrap().borrow().get_name());

    for (id, object) in sorted_lights {
      let is_selected = ui_manager.selected_object == SelectedObject::Light(*id);
      if ui.selectable_label(is_selected, object.borrow().get_name()).clicked() {
        ui_manager.selected_object = SelectedObject::Light(*id);
        scene.selected_object = ui_manager.selected_object.to_scene(&scene);
      }
    }
  }

  pub fn draw_light_properties(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene, selected_id: Uuid) {
    let Some(light_rc) = scene.lights_by_id.get(&selected_id).cloned() else {
      ui_manager.selected_object = SelectedObject::None;
      scene.selected_object = SceneSelectedObject::None;
      ui.label("No light selected");
      return;
    };

    let mut light = light_rc.borrow_mut();

    ui.heading("Name");
    ui.add(egui::TextEdit::singleline(light.get_name_mut()));
    ui.separator();
    ui.horizontal(|ui| {
      let delete_button = egui::Button::new("Delete Light").fill(egui::Color32::from_rgb(180, 40, 40));
      if ui.add(delete_button).clicked() {
        ui_manager.commands_queue.push(UICommand::DeleteObject(SelectedObject::Light(selected_id)));
      }

      if ui.button("Clone Object").clicked() {
        ui_manager.commands_queue.push(UICommand::CloneObject(SelectedObject::Light(selected_id)));
      }
    });
    ui.separator();

    let position = light.get_position_mut();
    ui.heading("Position");
    ui.horizontal(|ui| {
      ui.label("X: ");
      ui.add(egui::DragValue::new(&mut position.x).speed(0.1));
      ui.label("Y: ");
      ui.add(egui::DragValue::new(&mut position.y).speed(0.1));
      ui.label("Z: ");
      ui.add(egui::DragValue::new(&mut position.z).speed(0.1));
    });

    ui.heading("Light");
    let mut ambient = light.get_ambient().to_array();
    let mut diffuse = light.get_diffuse().to_array();
    let mut specular = light.get_specular().to_array();
    ui.horizontal(|ui| {
      ui.label("Ambient: ");
      if ui.color_edit_button_rgb(&mut ambient).changed() {
        light.set_ambient(Vec3::from_array(ambient));
      }
      ui.label("Diffuse: ");
      if ui.color_edit_button_rgb(&mut diffuse).changed() {
        light.set_diffuse(Vec3::from_array(diffuse));
      }
      ui.label("Specular: ");
      if ui.color_edit_button_rgb(&mut specular).changed() {
        light.set_specular(Vec3::from_array(specular));
      }
    });
  }
}

impl UIManager {
  pub fn draw_light_creation_options(&mut self, ui: &mut Ui) {
    if let CreatingObject::Light(props) = &mut self.creating_object {
      ui.heading("Light Type");
      ComboBox::from_label("Select the light type")
        .selected_text(format!("{:?}", props.primitive))
        .show_ui(ui, |ui| {
          ui.selectable_value(
            &mut props.primitive,
            LightType::PointLight,
            "Point Light",
          );
        });

      ui.separator();
      ui.heading("Properties");

      ui.horizontal(|ui| {
        ui.label("Ambient: ");
        ui.color_edit_button_rgb(&mut props.ambient);
      });
      ui.horizontal(|ui| {
        ui.label("Diffuse: ");
        ui.color_edit_button_rgb(&mut props.diffuse);
      });
      ui.horizontal(|ui| {
        ui.label("Specular: ");
        ui.color_edit_button_rgb(&mut props.specular);
      });

      ui.separator();
      ui.horizontal(|ui| {
        if ui.button("Create").clicked() {
          self.commands_queue.push(UICommand::CreateObject(self.creating_object.clone()));
          self.is_add_object_window_open = false;
        }
        if ui.button("Cancel").clicked() {
          self.is_add_object_window_open = false;
        }
      });
    }
  }
}
