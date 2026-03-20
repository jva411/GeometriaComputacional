use std::{cell::RefCell, path::PathBuf, rc::Rc};

use egui::{ComboBox, Ui};
use glam::Vec3;
use rfd::FileDialog;
use uuid::Uuid;

use crate::{objects::{geometry::points_cloud::PointsCloud, mesh::mesh::Mesh, object::{Object, ObjectType}, primitives::{cone::Cone, cube::Cube, cylinder::Cylinder, sphere::Sphere}}, scene::{scene::{Scene, SceneSelectedObject}, ui::ui::{CreatingObject, CreatingObjectType, SelectedObject, UICommand, UIManager}, window::Window}};


#[derive(Clone, Debug)]
pub struct NewObjectProperties {
  primitive: ObjectType,
  name: String,
  radius: f32,
  height: f32,
  subdivisions: u32,
  obj_path: Option<PathBuf>,
}

impl Default for NewObjectProperties {
  fn default() -> Self {
    NewObjectProperties {
      primitive: ObjectType::Cube,
      name: String::from("Cube"),
      radius: 0.5,
      height: 1.0,
      subdivisions: 30,
      obj_path: None,
    }
  }
}

impl Window {
  pub fn create_object(props: NewObjectProperties) -> Rc<RefCell<dyn Object>> {
    match props.primitive {
      ObjectType::Cube => Rc::new(RefCell::new(Cube::new(props.name))),
      ObjectType::Sphere => Rc::new(RefCell::new(Sphere::new(props.name, props.radius, props.subdivisions))),
      ObjectType::Cylinder => Rc::new(RefCell::new(Cylinder::new(props.name, props.radius, props.height, props.subdivisions))),
      ObjectType::Cone => Rc::new(RefCell::new(Cone::new(props.name, props.radius, props.height, props.subdivisions))),
      ObjectType::Mesh => Rc::new(RefCell::new(Mesh::new(props.name, props.obj_path.unwrap()))),
      _ => unimplemented!("ObjectType::{:?} creation not implemented yet", props.primitive),
    }
  }

  pub fn clone_object(&mut self, selected_id: Uuid) {
    let object = self.scene.objects_by_id.get(&selected_id);
    if object.is_none() {
      return
    }

    let object = object.unwrap().clone();
    let object = object.borrow();

    let new_object_rc = object.clone_rc_ref();
    let new_object_id = new_object_rc.borrow().get_id();
    {
      let mut new_object = new_object_rc.borrow_mut();
      new_object.set_name(format!("{} Copy", object.get_name()));
    }
    self.scene.add_object(new_object_rc);
    self.select_object(SelectedObject::Object(new_object_id));
  }

  pub fn create_points_cloud_from_object(&mut self, selected_id: Uuid) {
    let object = self.scene.objects_by_id.get(&selected_id);
    if object.is_none() {
      return
    }

    let object = object.unwrap().clone();
    let object = object.borrow();
    let n_samples = 1000;
    let cloud = object.generate_points_cloud_with_inner_samples(n_samples);
    if cloud.is_none() {
      return
    }

    let cloud = cloud.unwrap();
    let cloud_id = cloud.get_id();
    self.scene.add_object(Rc::new(RefCell::new(cloud)));
    self.select_object(SelectedObject::Object(cloud_id));
  }

  pub fn create_convex_hull_from_points_cloud(&mut self, selected_id: Uuid) {
    let object = self.scene.objects_by_id.get(&selected_id);
    if object.is_none() {
      return
    }

    let object = object.unwrap().clone();
    let object = object.borrow();
    if object.get_type() != ObjectType::PointsCloud {
      return
    }

    let cloud = object.as_any().downcast_ref::<PointsCloud>().unwrap();
    let hull = cloud.convex_hull();
    let hull_id = hull.get_id();
    self.scene.add_object(Rc::new(RefCell::new(hull)));
    self.select_object(SelectedObject::Object(hull_id));
  }

  pub fn draw_objects_list(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene) {
    if ui.button("Add Object").clicked() {
      ui_manager.is_add_object_window_open = true;
      ui_manager.creating_object = CreatingObject::default();
      ui_manager.creating_object_type = CreatingObjectType::Object;
    }
    ui.separator();

    let mut sorted_objects = scene.objects_by_id
      .iter()
      .collect::<Vec<_>>();

    sorted_objects.sort_by_key(|(id, _)| scene.objects_by_id.get(id).unwrap().borrow().get_name());

    for (id, object) in sorted_objects {
      let is_selected = ui_manager.selected_object == SelectedObject::Object(*id);
      if ui.selectable_label(is_selected, object.borrow().get_name()).clicked() {
        ui_manager.selected_object = SelectedObject::Object(*id);
        scene.selected_object = ui_manager.selected_object.to_scene(&scene);
      }
    }
  }

  pub fn draw_object_properties(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene, selected_id: Uuid) {
    let Some(object_rc) = scene.objects_by_id.get(&selected_id).cloned() else {
      ui_manager.selected_object = SelectedObject::None;
      scene.selected_object = SceneSelectedObject::None;
      ui.label("No object selected");
      return;
    };

    let mut object = object_rc.borrow_mut();

    ui.heading("Name");
    ui.add(egui::TextEdit::singleline(object.get_name_mut()));
    ui.separator();
    ui.horizontal(|ui| {
      let delete_button = egui::Button::new("Delete Object").fill(egui::Color32::from_rgb(180, 40, 40));
      if ui.add(delete_button).clicked() {
        ui_manager.commands_queue.push(UICommand::DeleteObject(SelectedObject::Object(selected_id)));
      }

      if ui.button("Clone Object").clicked() {
        ui_manager.commands_queue.push(UICommand::CloneObject(SelectedObject::Object(selected_id)));
      }
    });
    ui.separator();

    let transform = object.get_transform_mut();
    ui.heading("Translation");
    ui.horizontal(|ui| {
      ui.label("X: ");
      ui.add(egui::DragValue::new(&mut transform.translation.x).speed(0.1));
      ui.label("Y: ");
      ui.add(egui::DragValue::new(&mut transform.translation.y).speed(0.1));
      ui.label("Z: ");
      ui.add(egui::DragValue::new(&mut transform.translation.z).speed(0.1));
    });

    ui.heading("Rotation");
    ui.horizontal(|ui| {
      ui.label("Yaw: ");
      ui.add(egui::DragValue::new(&mut transform.rotation.yaw).speed(0.5));
      ui.label("Pitch: ");
      ui.add(egui::DragValue::new(&mut transform.rotation.pitch).speed(0.5));
      ui.label("Roll: ");
      ui.add(egui::DragValue::new(&mut transform.rotation.roll).speed(0.5));
    });

    ui.heading("Scale");
    ui.horizontal(|ui| {
      ui.label("X: ");
      ui.add(egui::DragValue::new(&mut transform.scale.x).speed(0.1));
      ui.label("Y: ");
      ui.add(egui::DragValue::new(&mut transform.scale.y).speed(0.1));
      ui.label("Z: ");
      ui.add(egui::DragValue::new(&mut transform.scale.z).speed(0.1));
    });
    ui.separator();

    let material = object.get_material_mut();
    let mut ambient = material.ambient.to_array();
    let mut diffuse = material.diffuse.to_array();
    let mut specular = material.specular.to_array();
    ui.heading("Material");
    ui.horizontal(|ui| {
      ui.label("Ambient: ");
      if ui.color_edit_button_rgb(&mut ambient).changed() {
        material.ambient = Vec3::from_array(ambient);
      }
      ui.label("Diffuse: ");
      if ui.color_edit_button_rgb(&mut diffuse).changed() {
        material.diffuse = Vec3::from_array(diffuse);
      }
      ui.label("Specular: ");
      if ui.color_edit_button_rgb(&mut specular).changed() {
        material.specular = Vec3::from_array(specular);
      }
    });
    ui.label("Shininess: ");
    ui.add(egui::DragValue::new(&mut material.shininess).range(0.0..=256.0).speed(0.1));
    ui.separator();

    if object.can_generate_points_cloud() {
      if ui.button("Points Cloud").clicked() {
        ui_manager.commands_queue.push(UICommand::CreatePointsCloud(SelectedObject::Object(selected_id)));
      }
    }

    if object.get_type() == ObjectType::PointsCloud {
      if ui.button("Convex Hull").clicked() {
        ui_manager.commands_queue.push(UICommand::CreateConvexHull(SelectedObject::Object(selected_id)));
      }
    }
  }
}

impl UIManager {
  pub fn draw_object_creation_options(&mut self, ui: &mut Ui) {
    if let CreatingObject::Object(props) = &mut self.creating_object {
      ui.heading("Primitive");
      ComboBox::from_label("Select the primitive")
        .selected_text(format!("{:?}", props.primitive))
        .show_ui(ui, |ui| {
          ui.selectable_value(
            &mut props.primitive,
            ObjectType::Sphere,
            "Sphere",
          );
          ui.selectable_value(
            &mut props.primitive,
            ObjectType::Cube,
            "Cube",
          );
          ui.selectable_value(
            &mut props.primitive,
            ObjectType::Cylinder,
            "Cylinder",
          );
          ui.selectable_value(
            &mut props.primitive,
            ObjectType::Cone,
            "Cone",
          );
          ui.selectable_value(
            &mut props.primitive,
            ObjectType::Mesh,
            "Mesh",
          );
        });

      ui.separator();
      ui.heading("Properties");
      ui.horizontal(|ui| {
        ui.label("Name: ");
        ui.text_edit_singleline(&mut props.name);
      });

      match props.primitive {
        ObjectType::Cube => {},
        ObjectType::Sphere => {
          ui.horizontal(|ui| {
            ui.label("Radius: ");
            ui.add(egui::DragValue::new(&mut props.radius).speed(0.1));
          });
          ui.horizontal(|ui| {
            ui.label("Subdivisions: ");
            ui.add(egui::DragValue::new(&mut props.subdivisions).range(0..=100).speed(1));
          });
        },
        ObjectType::Cylinder | ObjectType::Cone => {
          ui.horizontal(|ui| {
            ui.label("Radius: ");
            ui.add(egui::DragValue::new(&mut props.radius).speed(0.1));
          });
          ui.horizontal(|ui| {
            ui.label("Height: ");
            ui.add(egui::DragValue::new(&mut props.height).speed(0.1));
          });
          ui.horizontal(|ui| {
            ui.label("Subdivisions: ");
            ui.add(egui::DragValue::new(&mut props.subdivisions).range(4..=100).speed(1));
          });
        },
        ObjectType::Mesh => {
          ui.label("Object File: ");
          let placeholder = if let Some(path) = &props.obj_path {
            path.file_stem().unwrap().to_str().unwrap()
          } else {
            "Select File"
          };

          if ui.button(placeholder).clicked() {
            let path = FileDialog::new().add_filter("Obj", &["obj"]).pick_file();
            let path = path.unwrap();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            props.name = stem.to_string();
            props.obj_path = Some(path);
          }

          ui.horizontal(|ui| {
            ui.label("Scale");
            ui.add(egui::DragValue::new(&mut props.radius).range(0..=1).speed(0.001));
          });
        },
        _ => {
          unimplemented!();
        }
      }

      ui.separator();
      ui.horizontal(|ui| {
        let should_enable_creation = true;
        if ui.button("Create").clicked() {
          if should_enable_creation {
            self.commands_queue.push(UICommand::CreateObject(self.creating_object.clone()));
            self.is_add_object_window_open = false;
          }
        }
        if ui.button("Cancel").clicked() {
          self.is_add_object_window_open = false;
        }
      });
    }
  }
}
