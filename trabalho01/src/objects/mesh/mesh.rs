use std::path::PathBuf;

use glam::Vec3;
use parry3d::transformation::{convex_hull, vhacd::{VHACD, VHACDParameters}};
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::{geometry::points_cloud::PointsCloud, object::{Object, ObjectType}}, opengl::{ebo::EBO, program::Program, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform, vector::calculate_normals}};

pub struct Mesh {
  pub id: Uuid,
  pub name: String,

  pub transform: Transform,
  pub material: Material,

  pub vertices: Vec<Vec3>,
  pub normals: Vec<Vec3>,
  pub faces: Vec<u32>,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,
}

impl Mesh {
  pub fn new(name: String, vertices: Vec<Vec3>, normals: Vec<Vec3>, faces: Vec<u32>) -> Self {
    let normals = if normals.is_empty() {
      calculate_normals(&vertices, &faces)
    } else {
      normals
    };

    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();

    vao.bind();
    vbo.bind();
    ebo.bind();
    vbo.send_data(
      &vertices
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| [v.x, v.y, v.z, n.x, n.y, n.z])
        .flatten()
        .collect::<Vec<f32>>(),
    );
    ebo.send_data(&faces);

    let stride = 6 * SIZE_F32;
    vao.add_attribute(0, stride, 0);
    vao.add_attribute(1, stride, 3 * SIZE_F32);

    let mesh = Self {
      id: Uuid::new_v4(),
      name,

      transform: Transform::default(),
      material: Material::default(),

      vertices,
      normals,
      faces,

      vao,
      vbo,
      ebo,
    };

    return mesh;
  }

  pub fn from_obj_file(name: String, file_path: PathBuf, scale: f32) -> Self {
    let (models, _) = tobj::load_obj(
      file_path,
      &tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ..Default::default()
      },
    )
    .expect("Failed to load obj file");

    let mut vertices: Vec<Vec3> = Vec::new();
    let mut normals = Vec::new();
    let mut faces: Vec<u32> = Vec::new();

    for model in models {
      let mesh = &model.mesh;
      let vertex_offset = vertices.len() as u32;

      vertices.extend(
        mesh
          .positions
          .chunks_exact(3)
          .map(|p| Vec3::new(p[0], p[1], p[2])),
      );

      normals.extend(
        mesh
          .normals
          .chunks_exact(3)
          .map(|p| Vec3::new(p[0], p[1], p[2])),
      );

      faces.extend(mesh.indices.iter().map(|i| *i as u32 + vertex_offset));
    }

    if normals.is_empty() {
      normals = calculate_normals(&vertices, &faces);
    }

    let mut mesh = Mesh::new(name, vertices, normals, faces);
    mesh.transform.scale3f(scale, scale, scale);

    return mesh;
  }

  fn convex_decomposition(&self) -> Vec<Vec<usize>> {
    let indices: Vec<[u32; 3]> = self
      .faces
      .chunks(3)
      .map(|chunk| [chunk[0], chunk[1], chunk[2]])
      .collect();

    let params = VHACDParameters {
      max_convex_hulls: 1024,
      concavity: 0.001,
      resolution: 256,
      ..VHACDParameters::default()
    };

    let decomposition = VHACD::decompose(
      &params,
      &self.vertices,
      &indices,
      false,
    );

    let hulls = decomposition.compute_convex_hulls(1);

    if hulls.is_empty() {
      return vec![(0..self.vertices.len()).collect()];
    }

    let mut parts_planes: Vec<Vec<(Vec3, Vec3)>> = Vec::with_capacity(hulls.len());
    for (hull_verts, hull_indices) in &hulls {
      let mut planes = Vec::new();
      for tri in hull_indices {
        let a = hull_verts[tri[0] as usize];
        let b = hull_verts[tri[1] as usize];
        let c = hull_verts[tri[2] as usize];

        let normal = (b - a).cross(c - a).normalize_or_zero();
        planes.push((a, normal));
      }
      parts_planes.push(planes);
    }

    let max_dim = self.vertices
      .iter()
      .map(|v| v.x.abs().max(v.y.abs()).max(v.z.abs()))
      .max_by(|a, b| a.partial_cmp(b).unwrap())
      .unwrap();

    println!("max_dim: {}", max_dim);
    let epsilon_percentage = 0.05;
    let epsilon = max_dim * epsilon_percentage;
    let mut parts_indices = vec![Vec::new(); hulls.len()];
    for (vertex_idx, &point) in self.vertices.iter().enumerate() {
      let mut distances = Vec::with_capacity(parts_planes.len());

      for planes in &parts_planes {
        let mut max_dist = f32::MIN;
        for (a, normal) in planes {
          let dist = (point - *a).dot(*normal);
          if dist > max_dist {
            max_dist = dist;
          }
        }
        distances.push(max_dist);
      }

      let min_dist = distances.iter().cloned().fold(f32::MAX, f32::min);

      for (part_idx, &dist) in distances.iter().enumerate() {
        if dist <= min_dist + epsilon {
          parts_indices[part_idx].push(vertex_idx);
        }
      }
    }

    return parts_indices;
  }
}

impl Object for Mesh {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Mesh }

  fn tick(&mut self) { }

  fn draw(&self, program: &Program, base_transform: Option<Transform>) {
    self.vao.bind();
    self.vbo.bind();
    self.ebo.bind();

    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };
    model_transform.send_to_program(&program);
    self.material.send_to_program(&program);

    unsafe {
      gl::DrawElements(gl::TRIANGLES, self.faces.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
    }
  }

  fn clone(&self) -> Self {
    let mut mesh = Mesh::new(self.name.clone(), self.vertices.clone(), self.normals.clone(), self.faces.clone());
    mesh.transform = self.transform.clone();
    mesh.material = self.material.clone();
    return mesh;
  }

  fn ray_intersection(&self, _ray: Ray) -> Option<f32> {
    todo!()
  }

  fn can_generate_convex_hull(&self) -> bool { true }

  fn convex_hull(&self) -> Option<Mesh> {
    let parts_indices = self.convex_decomposition();

    let mut global_faces = Vec::new();

    for part_vertex_indices in parts_indices {
      if part_vertex_indices.len() < 4 { continue; }

      let local_vertices: Vec<Vec3> = part_vertex_indices.iter().map(|&i| self.vertices[i]).collect();
      let (convex_hull_vertices, convex_hull_faces) = convex_hull(&local_vertices);
      let mut local_to_global = Vec::new();
      for ch_vert in &convex_hull_vertices {
        if let Some(local_idx) = local_vertices.iter().position(|v| v == ch_vert) {
          local_to_global.push(part_vertex_indices[local_idx]);
        } else {
          local_to_global.push(0);
        }
      }

      for face in convex_hull_faces {
        global_faces.push(local_to_global[face[0] as usize] as u32);
        global_faces.push(local_to_global[face[1] as usize] as u32);
        global_faces.push(local_to_global[face[2] as usize] as u32);
      }
    }

    let mut mesh = Mesh::new(
      format!("{}_convex_hull", self.name),
      self.vertices.clone(),
      Vec::new(),
      global_faces,
    );

    mesh.transform = self.transform.clone();
    mesh.material = self.material.clone();

    return Some(mesh);
  }

  fn can_generate_points_cloud(&self) -> bool { true }
  fn generate_points_cloud(&self, _use_parry: bool) -> Option<PointsCloud> {
    let points = self.vertices.clone();
    let mut cloud = PointsCloud::new(format!("{}_points", self.name), points, vec![]);
    cloud.transform = self.transform.clone();
    return Some(cloud);
  }

  fn generate_points_cloud_with_inner_samples(&self, _inner_samples: u32, _use_parry: bool) -> Option<PointsCloud> {
    return self.generate_points_cloud(_use_parry);
  }
}

implement_transformable!(Mesh);
