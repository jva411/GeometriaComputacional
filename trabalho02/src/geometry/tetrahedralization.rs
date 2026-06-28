use glam::Vec3;
use std::{collections::{HashMap, HashSet}, fmt::Display};

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
  pub points: Vec<Vec3>,
  pub tetrahedrons: Vec<Tetrahedron>,
}

pub fn advancing_front(points: &Vec<Vec3>, hull_points: &Vec<usize>, hull_faces: &Vec<usize>) -> TetrahedralMesh {
  try_advancing_front(points, hull_points, hull_faces).unwrap()
}

pub fn try_advancing_front(points: &Vec<Vec3>, hull_points: &Vec<usize>, hull_faces: &Vec<usize>) -> Option<TetrahedralMesh> {
  println!("Tetrahedralization: points: {}, hull_points: {}, hull_faces: {}", points.len(), hull_points.len(), hull_faces.len());
  if points.len() < 4 || hull_faces.len() < 3 {
    return None;
  }

  let mut tetrahedrons = Vec::new();
  let mut active_front: HashMap<Face, Face> = HashMap::new();

  let mut generated_tetras: HashSet<[usize; 4]> = HashSet::new();

  for chunk in hull_faces.chunks_exact(3) {
    let face = Face(chunk[0], chunk[1], chunk[2]);
    let sorted_face = face.sorted();
    if active_front.contains_key(&sorted_face) {
      active_front.remove(&sorted_face);
    } else {
      active_front.insert(face.sorted(), face);
    }
  }

  let mut used_points: HashSet<usize> = hull_points.iter().cloned().collect();
  let all_points_indices: Vec<usize> = (0..points.len() as usize).collect();

  println!("active_front: {}", active_front.len());

  while let Some(&sorted_face) = active_front.keys().next() {
    let face = active_front.remove(&sorted_face).unwrap();

    let p1 = points[face.0 as usize];
    let p2 = points[face.1 as usize];
    let p3 = points[face.2 as usize];

    let mut best_point: Option<usize> = None;
    let mut min_distance = f32::MAX;

    for &point_index in &all_points_indices {
      if face.contains(point_index) {
        continue;
      }

      let mut tet_signature = [face.0, face.1, face.2, point_index];
      tet_signature.sort_unstable();
      if generated_tetras.contains(&tet_signature) {
        continue;
      }

      let candidate_pt = points[point_index as usize];
      let normal = (p2 - p1).cross(p3 - p1);
      let dot = normal.dot(candidate_pt - p1);

      if dot >= -EPSILON || (dot.abs() / 6.0) < EPSILON {
        continue;
      }

      if !is_valid_tetrahedron(points, &face, point_index, &active_front) {
        continue;
      }

      let radius = circumsphere_radius_sq(p1, p2, p3, candidate_pt);
      if is_sphere_empty(points, p1, p2, p3, candidate_pt, &active_front) {
        if radius < min_distance {
          min_distance = radius;
          best_point = Some(point_index);
        }
      }
    }

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
        if active_front.contains_key(&sorted) {
          active_front.remove(&sorted);
        } else {
          active_front.insert(sorted, new_face);
        }
      }
    }
  }

  Some(TetrahedralMesh {
    points: points.clone(),
    tetrahedrons,
  })
}

const EPSILON: f32 = 1e-8;

pub fn is_valid_tetrahedron(
  points: &Vec<Vec3>,
  base_face: &Face,
  candidate_idx: usize,
  front: &HashMap<Face, Face>,
) -> bool {
  return true;
  let p0 = points[base_face.0 as usize];
  let p1 = points[base_face.1 as usize];
  let p2 = points[base_face.2 as usize];
  let pc = points[candidate_idx as usize];

  for (i, p) in points.iter().enumerate() {
    let idx = i as usize;
    if idx == base_face.0 || idx == base_face.1 || idx == base_face.2 || idx == candidate_idx {
      continue;
    }

    if is_point_in_tetrahedron(*p, p0, p1, p2, pc) {
      return false;
    }
  }

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

  for existing_face in front.values() {
    if existing_face.0 == base_face.0 && existing_face.1 == base_face.1 && existing_face.2 == base_face.2 {
      continue;
    }

    let fp0 = points[existing_face.0 as usize];
    let fp1 = points[existing_face.1 as usize];
    let fp2 = points[existing_face.2 as usize];

    for &(start, end) in &new_edges {
      if segment_intersects_triangle_interior(start, end, fp0, fp1, fp2) {
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
        if segment_intersects_triangle_interior(start, end, v0, v1, v2) {
          return false;
        }
      }
    }
  }

  true
}

fn is_point_in_tetrahedron(p: Vec3, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> bool {
  fn same_side(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, p: Vec3) -> bool {
    let normal = (v2 - v1).cross(v3 - v1);
    let dot_v4 = normal.dot(v4 - v1);
    let dot_p = normal.dot(p - v1);

    (dot_v4 * dot_p) > EPSILON
  }

  same_side(a, b, c, d, p) &&
  same_side(a, b, d, c, p) &&
  same_side(a, c, d, b, p) &&
  same_side(b, c, d, a, p)
}

fn segment_intersects_triangle_interior(p: Vec3, q: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> bool {
  let edge1 = v1 - v0;
  let edge2 = v2 - v0;
  let ray_dir = q - p;

  let h = ray_dir.cross(edge2);
  let a_det = edge1.dot(h);

  if a_det > -EPSILON && a_det < EPSILON {
    return false;
  }

  let f = 1.0 / a_det;
  let s = p - v0;
  let u = f * s.dot(h);

  if u < EPSILON || u > (1.0 - EPSILON) {
    return false;
  }

  let q_vec = s.cross(edge1);
  let v = f * ray_dir.dot(q_vec);

  if v < EPSILON || (u + v) > (1.0 - EPSILON) {
    return false;
  }

  let t = f * edge2.dot(q_vec);

  t > EPSILON && t < (1.0 - EPSILON)
}

fn circumsphere_radius_sq(p1: Vec3, p2: Vec3, p3: Vec3, p4: Vec3) -> f32 {
  let a = p2 - p1;
  let b = p3 - p1;
  let c = p4 - p1;
  let det = 2.0 * a.dot(b.cross(c));
  if det.abs() < EPSILON { return f32::MAX; }

  let center = (a.length_squared() * b.cross(c) +
    b.length_squared() * c.cross(a) +
    c.length_squared() * a.cross(b)) / det;

  return center.length_squared();
}

pub fn is_sphere_empty(
  points: &Vec<Vec3>,
  p1: Vec3, p2: Vec3, p3: Vec3, p4: Vec3,
  _front: &HashMap<Face, Face>
) -> bool {
  let a = p2 - p1;
  let b = p3 - p1;
  let c = p4 - p1;

  let det = 2.0 * a.dot(b.cross(c));

  if det.abs() < EPSILON {
    return false;
  }

  let center = (b.cross(c) * a.length_squared() +
    c.cross(a) * b.length_squared() +
    a.cross(b) * c.length_squared()) / det;

  let circumcenter = p1 + center;
  let radius_sq = center.length_squared();

  for &p in points {
    if p == p1 || p == p2 || p == p3 || p == p4 {
      continue;
    }

    let dist_sq = (p - circumcenter).length_squared();

    if dist_sq < radius_sq - 1e-5 {
      return false;
    }
  }

  true
}
