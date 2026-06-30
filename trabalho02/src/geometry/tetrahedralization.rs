use std::{collections::{HashSet, VecDeque}, fmt::Display};

use glam::{DVec3, Vec3};

use crate::geometry::octree::{AABB, PointOctree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Face(pub usize, pub usize, pub usize);

impl Face {
  pub fn sorted(&self) -> Self {
    let mut arr = [self.0, self.1, self.2];
    arr.sort_unstable();
    Face(arr[0], arr[1], arr[2])
  }

  pub fn contains(&self, point_idx: usize) -> bool {
    self.0 == point_idx || self.1 == point_idx || self.2 == point_idx
  }
}

impl Display for Face {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}, {}, {})", self.0, self.1, self.2)
  }
}

#[derive(Debug, Clone)]
pub struct Tetrahedron(pub usize, pub usize, pub usize, pub usize);

#[derive(Debug)]
pub struct TetrahedralMesh {
  pub points: Vec<DVec3>,
  pub tetrahedrons: Vec<Tetrahedron>,
}

pub fn try_advancing_front(points: &Vec<Vec3>, hull_points: &Vec<usize>, hull_faces: &Vec<usize>) -> Option<TetrahedralMesh> {
  println!("Tetrahedralization: points: {}, hull_points: {}, hull_faces: {}", points.len(), hull_points.len(), hull_faces.len());
  if points.len() < 4 || hull_faces.len() < 3 {
    return None;
  }

  let points = &points.iter().map(Vec3::as_dvec3).collect::<Vec<_>>();

  let mut min_bound = DVec3::splat(f64::MAX);
  let mut max_bound = DVec3::splat(f64::MIN);
  for &p in points {
    min_bound = min_bound.min(p);
    max_bound = max_bound.max(p);
  }

  let bounds = AABB { min: min_bound - 0.01, max: max_bound + 0.01 };
  let all_points_indices: Vec<usize> = (0..points.len()).collect();
  let point_octree = PointOctree::new(points, all_points_indices.clone(), bounds, 0, 7);

  let mesh_scale = (bounds.max - bounds.min).length().max(1e-6);
  let det_epsilon = (mesh_scale * mesh_scale * mesh_scale) * 1e-12;

  let max_tetrahedrons = (points.len() * 20).max(1000);
  let log_interval: u64 = 2000;
  let mut iterations: u64 = 0;
  let mut aborted = false;

  let mut tetrahedrons = Vec::new();
  let mut active_front_queue: VecDeque<Face> = VecDeque::new();
  let mut active_front_set: HashSet<Face> = HashSet::new();
  let mut generated_tetras: HashSet<[usize; 4]> = HashSet::new();

  let mut initial_faces = Vec::new();
  for chunk in hull_faces.chunks_exact(3) {
    initial_faces.push(Face(chunk[0], chunk[1], chunk[2]));
  }

  initial_faces.sort_by(|a, b| {
    let area_a = face_area(points, a);
    let area_b = face_area(points, b);
    area_a.partial_cmp(&area_b).unwrap_or(std::cmp::Ordering::Equal)
  });

  for face in initial_faces {
    let sorted_face = face.sorted();
    if active_front_set.contains(&sorted_face) {
      active_front_set.remove(&sorted_face);
    } else {
      active_front_set.insert(sorted_face);
      active_front_queue.push_back(face);
    }
  }

  let mut used_points: HashSet<usize> = hull_points.iter().cloned().collect();

  println!("active_front inicial: {}", active_front_queue.len());

  while let Some(face) = active_front_queue.pop_front() {
    iterations += 1;

    if iterations % log_interval == 0 {
      println!(
        "Tetrahedralization: iteração {}, fila ativa={}, tetraedros gerados={}",
        iterations, active_front_queue.len(), tetrahedrons.len()
      );
    }

    if tetrahedrons.len() >= max_tetrahedrons {
      println!(
        "AVISO: Tetrahedralization abortada pela trava de segurança ({} tetraedros, fila restante={}). Possível front não convergente.",
        tetrahedrons.len(), active_front_queue.len() + 1
      );
      aborted = true;
      break;
    }

    let sorted_face = face.sorted();
    if !active_front_set.contains(&sorted_face) {
      continue;
    }

    active_front_set.remove(&sorted_face);

    let p1 = points[face.0 as usize];
    let p2 = points[face.1 as usize];
    let p3 = points[face.2 as usize];

    let normal = (p2 - p1).cross(p3 - p1);

    let centroid = (p1 + p2 + p3) / 3.0;
      let max_edge = (p1 - p2).length().max((p2 - p3).length()).max((p3 - p1).length());

      let max_search_limit = (bounds.max - bounds.min).length();
      let mut search_radius = (max_edge * 2.0).max(1e-8).min(max_search_limit);

      let mut best_point: Option<usize> = None;
      let mut fallback_point: Option<(usize, f64)> = None;
      let mut points_in_radius = Vec::new();

      while search_radius <= max_search_limit {
        points_in_radius.clear();
        point_octree.query_sphere(points, centroid, search_radius * search_radius, &mut points_in_radius);
        let mut candidates = Vec::new();
        for &point_index in &points_in_radius {
          if face.contains(point_index) { continue; }

          let mut tet_signature = [face.0, face.1, face.2, point_index];
          tet_signature.sort_unstable();
          if generated_tetras.contains(&tet_signature) { continue; }

          let candidate_pt = points[point_index];
          let to_candidate = candidate_pt - p1;

          let face_normal_normalized = normal.normalize();

          let height = face_normal_normalized.dot(to_candidate);

          let height_tolerance = max_edge * 0.005;

          if height > -height_tolerance {
            continue;
          }

          let a = p2 - p1;
          let b = p3 - p1;
          let c = candidate_pt - p1;
          let det = 2.0 * a.dot(b.cross(c));
          if det.abs() <= det_epsilon { continue; }

          let center_offset = (
            a.length_squared() * b.cross(c) +
            b.length_squared() * c.cross(a) +
            c.length_squared() * a.cross(b)
          ) / det;
          let radius_sq = center_offset.length_squared();
          let circumcenter = p1 + center_offset;

          candidates.push((point_index, radius_sq, circumcenter));
        }

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (point_index, radius_sq, circumcenter) in candidates {
          if !is_valid_tetrahedron(points, &face, point_index, &active_front_set, det_epsilon) {
            continue;
          }

          if fallback_point.map_or(true, |(_, best_radius_sq)| radius_sq < best_radius_sq) {
            fallback_point = Some((point_index, radius_sq));
          }

          if is_sphere_empty_octree(&point_octree, points, circumcenter, radius_sq, [face.0, face.1, face.2, point_index]) {
            best_point = Some(point_index);
            break;
          }
        }

        if best_point.is_some() {
          break;
        }

        search_radius *= 2.0;
    }

    let best_point = best_point.or_else(|| {
      fallback_point.map(|(point_index, _)| {
        point_index
      })
    });

    if let Some(point_index) = best_point {
      tetrahedrons.push(Tetrahedron(face.0, face.1, face.2, point_index));
      used_points.insert(point_index);

      let mut tet_signature = [face.0, face.1, face.2, point_index];
      tet_signature.sort_unstable();
      generated_tetras.insert(tet_signature);

      let new_faces = [
        Face(face.0, face.1, point_index),
        Face(face.1, face.2, point_index),
        Face(face.2, face.0, point_index),
      ];

      for new_face in new_faces {
        let sorted = new_face.sorted();

        if active_front_set.contains(&sorted) {
          active_front_set.remove(&sorted);
        } else {
          active_front_set.insert(sorted);
          active_front_queue.push_back(new_face);
        }
      }
    }
  }

  if aborted {
    println!(
      "Tetrahedralization: finalizada por abort de segurança após {} iterações, {} tetraedros gerados, {} faces restantes no front.",
      iterations, tetrahedrons.len(), active_front_queue.len() + active_front_set.len()
    );
  } else {
    println!(
      "Tetrahedralization: front convergiu após {} iterações, {} tetraedros gerados.",
      iterations, tetrahedrons.len()
    );
  }

  Some(TetrahedralMesh {
    points: points.clone(),
    tetrahedrons,
  })
}

pub fn is_valid_tetrahedron(
  points: &Vec<DVec3>,
  base_face: &Face,
  candidate_idx: usize,
  front: &HashSet<Face>,
  epsilon: f64,
) -> bool {
  let p0 = points[base_face.0];
  let p1 = points[base_face.1];
  let p2 = points[base_face.2];
  let pc = points[candidate_idx];

  let tet_min = p0.min(p1).min(p2).min(pc);
  let tet_max = p0.max(p1).max(p2).max(pc);

  let new_edges = [
    (pc, p0),
    (pc, p1),
    (pc, p2),
  ];

  let new_faces = [
    (pc, p0, p1),
    (pc, p1, p2),
    (pc, p2, p0),
  ];

  let sorted_base = base_face.sorted();

  for existing_face in front.iter() {
    if *existing_face == sorted_base {
      continue;
    }

    let fp0 = points[existing_face.0];
    let fp1 = points[existing_face.1];
    let fp2 = points[existing_face.2];

    let face_min = fp0.min(fp1).min(fp2);
    let face_max = fp0.max(fp1).max(fp2);

    let pad = 1e-8;
    if tet_max.x < face_min.x - pad || tet_min.x > face_max.x + pad ||
      tet_max.y < face_min.y - pad || tet_min.y > face_max.y + pad ||
      tet_max.z < face_min.z - pad || tet_min.z > face_max.z + pad {
      continue;
    }

    for &(start, end) in &new_edges {
      if shares_vertex(start, end, fp0, fp1, fp2) { continue; }
      if segment_intersects_triangle_interior(start, end, fp0, fp1, fp2, epsilon) {
        return false;
      }
    }

    let front_edges = [
      (fp0, fp1),
      (fp1, fp2),
      (fp2, fp0),
    ];

    for &(start, end) in &front_edges {
      for &(v0, v1, v2) in &new_faces {
        if shares_vertex(start, end, v0, v1, v2) { continue; }
        if segment_intersects_triangle_interior(start, end, v0, v1, v2, epsilon) {
          return false;
        }
      }
    }
  }

  true
}


fn segment_intersects_triangle_interior(p: DVec3, q: DVec3, v0: DVec3, v1: DVec3, v2: DVec3, degeneracy_epsilon: f64) -> bool {
  const PARAM_EPSILON: f64 = 1e-10;

  let edge1 = v1 - v0;
  let edge2 = v2 - v0;
  let ray_dir = q - p;

  let h = ray_dir.cross(edge2);
  let a_det = edge1.dot(h);

  if a_det > -degeneracy_epsilon && a_det < degeneracy_epsilon {
    return false;
  }

  let f = 1.0 / a_det;
  let s = p - v0;
  let u = f * s.dot(h);

  if u < PARAM_EPSILON || u > (1.0 - PARAM_EPSILON) {
    return false;
  }

  let q_vec = s.cross(edge1);
  let v = f * ray_dir.dot(q_vec);

  if v < PARAM_EPSILON || (u + v) > (1.0 - PARAM_EPSILON) {
    return false;
  }

  let t = f * edge2.dot(q_vec);

  t > PARAM_EPSILON && t < (1.0 - PARAM_EPSILON)
}

pub fn is_sphere_empty_octree(
  octree: &PointOctree,
  points: &Vec<DVec3>,
  circumcenter: DVec3,
  radius_sq: f64,
  ignore_indices: [usize; 4]
) -> bool {
  let mut points_in_sphere = Vec::new();

  octree.query_sphere(points, circumcenter, radius_sq * 0.999, &mut points_in_sphere);

  for idx in points_in_sphere {
    if !ignore_indices.contains(&idx) {
      return false;
    }
  }

  true
}

fn shares_vertex(start: DVec3, end: DVec3, v0: DVec3, v1: DVec3, v2: DVec3) -> bool {
  start == v0 || start == v1 || start == v2 || end == v0 || end == v1 || end == v2
}

fn face_area(points: &Vec<DVec3>, face: &Face) -> f64 {
  let p0 = points[face.0];
  let p1 = points[face.1];
  let p2 = points[face.2];
  0.5 * (p1 - p0).cross(p2 - p0).length()
}
