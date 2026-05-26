use std::{collections::{HashMap, HashSet}, fs::File, io::{BufWriter, Write}, path::PathBuf};

use glam::Vec3;
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::{mesh::mesh::Mesh, object::{Object, ObjectType}}, opengl::{renderer::Renderer, vao::VAO, vbo::VBO} , utils::{material::Material, transform::Transform}};

pub struct ConvexHull {
  pub id: Uuid,
  pub name: String,

  pub transform: Transform,
  pub material: Material,

  pub mesh: Mesh,
  pub hull_vertices: HashSet<usize>,
  pub hull_parts: Vec<Vec<u32>>,

  pub render_points: bool,
  pub render_mesh: bool,
  vao: VAO,
  vbo: VBO,
}

impl ConvexHull {
  pub fn new(name: String, mesh: Mesh, hull_vertices: HashSet<usize>) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    vao.bind();
    vbo.bind();

    let inner_points_color = Vec3::new(0.0, 1.0, 0.0);
    let hull_points_color = Vec3::new(1.0, 0.0, 0.0);
    let vertex_data = mesh.vertices.iter()
      .enumerate()
      .flat_map(|(i, point)| {
        let color = if hull_vertices.contains(&i) { hull_points_color } else { inner_points_color };
        vec![point.x, point.y, point.z, color.x, color.y, color.z]
      })
      .collect::<Vec<f32>>();

    vbo.send_data(&vertex_data);
    let stride = (6 * std::mem::size_of::<f32>()) as u32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, (3 * std::mem::size_of::<f32>()) as u32);

    let hull_parts = vec![mesh.faces.clone()];

    let mesh = Self {
      id: Uuid::new_v4(),
      name,

      transform: Transform::default(),
      material: Material::default(),

      mesh,
      hull_vertices,
      hull_parts,

      render_points: false,
      render_mesh: true,
      vao,
      vbo,
    };

    return mesh;
  }

  pub fn with_parts(name: String, mesh: Mesh, hull_vertices: HashSet<usize>, hull_parts: Vec<Vec<u32>>) -> Self {
    let mut hull = Self::new(name, mesh, hull_vertices);
    hull.hull_parts = hull_parts;
    return hull;
  }

  pub fn export_to_obj(&self, file_path: PathBuf) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "o {}", self.name)?;
    let mut vertex_offset = 1;
    for (group_index, group_faces) in self.hull_parts.iter().enumerate() {
      writeln!(writer, "g Parte_{}", group_index + 1)?;

      let mut unique_verts_map = HashMap::new();
      let mut local_vertices = Vec::new();

      for &original_idx in group_faces {
        if !unique_verts_map.contains_key(&original_idx) {
          unique_verts_map.insert(original_idx, local_vertices.len());
          local_vertices.push(self.mesh.vertices[original_idx as usize]);
        }
      }

      for v in &local_vertices {
        writeln!(writer, "v {:.6} {:.6} {:.6}", v.x, v.y, v.z)?;
      }

      for chunk in group_faces.chunks(3) {
        if chunk.len() == 3 {
          let v1 = unique_verts_map[&chunk[0]] + vertex_offset;
          let v2 = unique_verts_map[&chunk[1]] + vertex_offset;
          let v3 = unique_verts_map[&chunk[2]] + vertex_offset;
          writeln!(writer, "f {} {} {}", v1, v2, v3)?;
        }
      }

      writer.flush()?;
      vertex_offset += local_vertices.len();
    }

    writer.flush()?;

    Ok(())
  }
}

impl Object for ConvexHull {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::ConvexHull }

  fn tick(&mut self) { }

  fn draw(&self, renderer:  &mut Renderer, base_transform: Option<Transform>) {
    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };

    if self.render_mesh {
      self.mesh.draw(renderer, Some(model_transform.clone()));
    }

    let program = &renderer.current_program;
    self.vao.bind();
    self.vbo.bind();
    model_transform.send_to_program(program);
    unsafe {
      if self.render_points {
        program.set_uniform_bool("uSimplex", true).unwrap();
        gl::Disable(gl::DEPTH_TEST);
        gl::DrawArrays(gl::POINTS, 0, self.mesh.vertices.len() as i32);
        gl::Enable(gl::DEPTH_TEST);
        program.set_uniform_bool("uSimplex", false).unwrap();
      }
    }
  }

  fn clone(&self) -> Self {
    let mut mesh = Self::with_parts(self.name.clone(), self.mesh.clone(), self.hull_vertices.clone(), self.hull_parts.clone());
    mesh.transform = self.transform.clone();
    mesh.material = self.material.clone();
    return mesh;
  }
}

implement_transformable!(ConvexHull);
