use std::{cell::RefCell, path::PathBuf, rc::Rc};

use egui::{ComboBox, Ui};
use glam::Vec3;
use rfd::FileDialog;
use uuid::Uuid;

use crate::{objects::{geometry::{convex_hull::ConvexHull, tetrahedron::TetrahedronObject}, mesh::mesh::Mesh, object::{ConvexHullProps, Object, ObjectType}, primitives::{cone::Cone, cube::Cube, cylinder::Cylinder, planes::square::Square, sphere::Sphere}}, scene::{scene::Scene, ui::ui::{CreateConvexHullMode, CreatingObject, CreatingObjectType, SelectedObject, UICommand, UIManager}, window::Window}};


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
      radius: 1.0,
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
      ObjectType::Mesh => Rc::new(RefCell::new(Mesh::from_obj_file(props.name, props.obj_path.unwrap(), props.radius))),
      ObjectType::Square => Rc::new(RefCell::new(Square::new(props.name, Vec3::ZERO, Vec3::NEG_Z))),
      #[allow(unreachable_patterns)]
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

  pub fn save_object(&mut self, selected_id: Uuid, path: PathBuf) {
    let object = self.scene.objects_by_id.get(&selected_id);
    if object.is_none() {
      return
    }

    let object = object.unwrap().clone();
    let object = object.borrow();

    match object.get_type() {
      ObjectType::ConvexHull => {
        let hull = object.as_any().downcast_ref::<ConvexHull>().unwrap();
        let _ = hull.export_to_obj(path);
      }
      ObjectType::Tetrahedron => {
        let tetrahedron = object.as_any().downcast_ref::<TetrahedronObject>().unwrap();
        let _ = tetrahedron.export_to_obj(path);
      }
      _ => {}
    };
  }

  pub fn create_convex_hull(&mut self, selected_id: Uuid, mode: CreateConvexHullMode) {
    let object = self.scene.objects_by_id.get(&selected_id);
    if object.is_none() {
      return
    }

    let object = object.unwrap().clone();
    let object = object.borrow();
    let n_samples = 100;
    let hull = match mode {
      CreateConvexHullMode::Default => object.convex_hull(false),
      CreateConvexHullMode::RandomPoints => object.convex_hull_with_inner_samples(ConvexHullProps::RandomPoints(n_samples)),
      CreateConvexHullMode::OctreePoints => object.convex_hull_with_inner_samples(ConvexHullProps::OctreePoints),
    };
    if hull.is_none() {
      return
    }

    let hull = hull.unwrap();
    let hull_id = hull.get_id();
    self.scene.add_object(Rc::new(RefCell::new(hull)));
    self.select_object(SelectedObject::Object(hull_id));
  }

  pub fn tetrahedralization(&mut self, selected_id: Uuid) {
    let object = self.scene.objects_by_id.get(&selected_id);
    if object.is_none() {
      return
    }

    let object = object.unwrap().clone();
    let object = object.borrow();
    if object.get_type() != ObjectType::ConvexHull {
      return
    }

    let hull = object.as_any().downcast_ref::<ConvexHull>().unwrap();
    let tetrahedron_object = hull.tetrahedralization();
    if tetrahedron_object.is_none() {
      return
    }

    let tetrahedron_object = tetrahedron_object.unwrap();
    let tetrahedron_object_id = tetrahedron_object.get_id();
    self.scene.add_object(Rc::new(RefCell::new(tetrahedron_object)));
    self.select_object(SelectedObject::Object(tetrahedron_object_id));
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
      }
    }
  }

  pub fn draw_object_properties(ui: &mut Ui, ui_manager: &mut UIManager, scene: &mut Scene, selected_id: Uuid) {
    let Some(object_rc) = scene.objects_by_id.get(&selected_id).cloned() else {
      ui_manager.selected_object = SelectedObject::None;
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

      if object.get_type() == ObjectType::ConvexHull || object.get_type() == ObjectType::Tetrahedron {
        if ui.button("Save OBJ").clicked() {
          let path = FileDialog::new()
            .add_filter("OBJ", &["obj"])
            .set_file_name(format!("{}.obj", object.get_name()))
            .save_file();

          if let Some(path) = path {
            ui_manager.commands_queue.push(UICommand::SaveObject(SelectedObject::Object(selected_id), path));
          }
        }
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

    if object.can_generate_convex_hull() {
      if ui.button("Convex Hull").clicked() {
        ui_manager.commands_queue.push(UICommand::CreateConvexHull(SelectedObject::Object(selected_id), CreateConvexHullMode::Default));
      }

      if object.get_type() != ObjectType::Mesh {
        if ui.button("Convex Hull (Random)").clicked() {
          ui_manager.commands_queue.push(UICommand::CreateConvexHull(SelectedObject::Object(selected_id), CreateConvexHullMode::RandomPoints));
        }
      }

      if ui.button("Convex Hull (Octree)").clicked() {
        ui_manager.commands_queue.push(UICommand::CreateConvexHull(SelectedObject::Object(selected_id), CreateConvexHullMode::OctreePoints));
      }
    }

    if object.get_type() == ObjectType::ConvexHull {
      let hull = object.as_any_mut().downcast_mut::<ConvexHull>().unwrap();
      ui.heading("Convex Hull");
      ui.checkbox(&mut hull.render_points, "View Points");
      ui.checkbox(&mut hull.render_mesh, "View Mesh");

      if ui.button("Triangulate").clicked() {
        ui_manager.commands_queue.push(UICommand::Tetrahedralization(SelectedObject::Object(selected_id)));
      }
    }

    if object.get_type() == ObjectType::Tetrahedron {
      let object = object.as_any_mut().downcast_mut::<TetrahedronObject>().unwrap();
      ui.label("Shrink: ");
      ui.add(egui::DragValue::new(&mut object.shrink).range(0.0..=0.99).speed(0.01));

      ui.checkbox(&mut object.render_points, "View Points");
      ui.checkbox(&mut object.render_mesh, "View Mesh");
      ui.checkbox(&mut object.render_wireframe, "Wireframe");
    }

    if object.get_type() == ObjectType::Mesh {
      let object = object.as_any_mut().downcast_mut::<Mesh>().unwrap();
      ui.checkbox(&mut object.render_points, "View Points");
      ui.checkbox(&mut object.render_mesh, "View Mesh");
      ui.checkbox(&mut object.render_wireframe, "Wireframe");
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
          if ui.selectable_value(
            &mut props.primitive,
            ObjectType::Sphere,
            "Sphere",
          ).clicked() {
            props.name = String::from("Sphere");
          };
          if ui.selectable_value(
            &mut props.primitive,
            ObjectType::Cube,
            "Cube",
          ).clicked() {
            props.name = String::from("Cube");
          };
          if ui.selectable_value(
            &mut props.primitive,
            ObjectType::Cylinder,
            "Cylinder",
          ).clicked() {
            props.name = String::from("Cylinder");
          };
          if ui.selectable_value(
            &mut props.primitive,
            ObjectType::Cone,
            "Cone",
          ).clicked() {
            props.name = String::from("Cone");
          };
          if ui.selectable_value(
            &mut props.primitive,
            ObjectType::Mesh,
            "Mesh",
          ).clicked() {
            props.name = String::from("Mesh");
          };
          if ui.selectable_value(
            &mut props.primitive,
            ObjectType::Square,
            "Square",
          ).clicked() {
            props.name = String::from("Square");
          };
        });

      ui.separator();
      ui.heading("Properties");
      ui.horizontal(|ui| {
        ui.label("Name: ");
        ui.text_edit_singleline(&mut props.name);
      });

      match props.primitive {
        ObjectType::Cube => {}
        ObjectType::Sphere => {
          ui.horizontal(|ui| {
            ui.label("Radius: ");
            ui.add(egui::DragValue::new(&mut props.radius).speed(0.1));
          });
          ui.horizontal(|ui| {
            ui.label("Subdivisions: ");
            ui.add(egui::DragValue::new(&mut props.subdivisions).range(0..=100).speed(1));
          });
        }
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
        }
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
            ui.add(egui::DragValue::new(&mut props.radius).range(0..=usize::MAX).speed(0.001));
          });
        }
        ObjectType::Square => {
          ui.horizontal(|ui| {
            ui.label("Scale");
            ui.add(egui::DragValue::new(&mut props.radius).range(0..=1).speed(0.001));
          });
        }
        #[allow(unreachable_patterns)]
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
