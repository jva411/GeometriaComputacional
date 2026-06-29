use std::{collections::{HashMap, HashSet}, fs::File, io::{BufWriter, Write}, path::PathBuf};

use glam::Vec3;
use uuid::Uuid;

use crate::{geometry::{octree::OctreeNode, tetrahedralization::try_advancing_front}, implement_partial_Object, implement_transformable, objects::{geometry::tetrahedron::TetrahedronObject, mesh::mesh::Mesh, object::{Object, ObjectType}}, opengl::{renderer::Renderer, vao::VAO, vbo::VBO}, utils::{material::Material, transform::Transform}};

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

  pub fn tetrahedralization(&self, use_octree: bool) -> Option<TetrahedronObject> {
    let mut parts = Vec::new();

    let points = if use_octree {
      let octree = OctreeNode::generate_from_mesh(&self.mesh.vertices, &self.mesh.faces, 4);
      let octree_new_points = octree.get_leaves_centroids(&self.mesh.vertices, &self.mesh.faces);
      let avg_edge = average_edge_length(&self.mesh.vertices, &self.hull_parts);
      let min_dist = (avg_edge * 0.25).max(1e-5);

      let mut all_points = self.mesh.vertices.clone();
      let filtered_octree_points = filter_close_points(&all_points, &octree_new_points, min_dist);
      println!(
        "Octree: {} pontos candidatos, {} aceitos após filtro de proximidade (min_dist={:.6})",
        octree_new_points.len(), filtered_octree_points.len(), min_dist
      );
      all_points.extend(filtered_octree_points);
      all_points
      // self.mesh.vertices.clone()
    } else {
      self.mesh.vertices.clone()
    };

    for part_faces in &self.hull_parts {
      if part_faces.len() < 3 {
        continue;
      }

      let mut unique_verts = HashSet::new();
      for &idx in part_faces {
        unique_verts.insert(idx as usize);
      }
      let hull_points: Vec<usize> = unique_verts.into_iter().collect();
      let hull_faces: Vec<usize> = part_faces.iter().map(|&idx| idx as usize).collect();

      if let Some(mesh) = try_advancing_front(&points, &hull_points, &hull_faces) {
        if !mesh.tetrahedrons.is_empty() {
          println!("{} tetrahedrons", mesh.tetrahedrons.len());
          parts.push(crate::objects::geometry::tetrahedron::TetrahedronPart {
            points: mesh.points,
            tetrahedrons: mesh.tetrahedrons,
          });
        } else {
          println!("No tetrahedrons created");
        }
      }
    }

    if parts.is_empty() {
      return None;
    }

    let mut tetra_obj = TetrahedronObject::new(
      format!("{}_tetrahedralized", self.name),
      parts,
    );

    tetra_obj.transform = self.transform.clone();
    tetra_obj.material = self.material.clone();

    Some(tetra_obj)
  }
}

impl Object for ConvexHull {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::ConvexHull }

  fn tick(&mut self) { }

  fn draw(&self, renderer: &mut Renderer, base_transform: Option<Transform>) {
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



fn average_edge_length(vertices: &[Vec3], hull_parts: &[Vec<u32>]) -> f32 {
  let mut total = 0.0f32;
  let mut count = 0u32;

  for part in hull_parts {
    for chunk in part.chunks_exact(3) {
      let a = vertices[chunk[0] as usize];
      let b = vertices[chunk[1] as usize];
      let c = vertices[chunk[2] as usize];
      total += (a - b).length() + (b - c).length() + (c - a).length();
      count += 3;
    }
  }

  if count == 0 { 1.0 } else { total / count as f32 }
}

fn filter_close_points(existing: &[Vec3], candidates: &[Vec3], min_dist: f32) -> Vec<Vec3> {
  if min_dist <= 0.0 {
    return candidates.to_vec();
  }

  let cell_size = min_dist;
  let cell_of = |p: Vec3| -> (i64, i64, i64) {
    (
      (p.x / cell_size).floor() as i64,
      (p.y / cell_size).floor() as i64,
      (p.z / cell_size).floor() as i64,
    )
  };

  let mut grid: HashMap<(i64, i64, i64), Vec<Vec3>> = HashMap::new();
  for &p in existing {
    grid.entry(cell_of(p)).or_insert_with(Vec::new).push(p);
  }

  let min_dist_sq = min_dist * min_dist;
  let mut accepted = Vec::new();

  for &p in candidates {
    let (cx, cy, cz) = cell_of(p);
    let mut too_close = false;

    'neighbors: for dx in -1..=1 {
      for dy in -1..=1 {
        for dz in -1..=1 {
          if let Some(cell_points) = grid.get(&(cx + dx, cy + dy, cz + dz)) {
            for &other in cell_points {
              if (other - p).length_squared() < min_dist_sq {
                too_close = true;
                break 'neighbors;
              }
            }
          }
        }
      }
    }

    if !too_close {
      grid.entry((cx, cy, cz)).or_insert_with(Vec::new).push(p);
      accepted.push(p);
    }
  }

  accepted
}
