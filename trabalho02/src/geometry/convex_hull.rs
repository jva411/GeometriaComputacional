use glam::Vec3;


struct HullFace {
  vertices: [usize; 3],
  visible_points: Vec<usize>,
  max_distant_point: Option<HullPoint>,
}


struct HullPoint {
  point: usize,
  pseudo_volume: f32,
}


impl HullFace {
  pub fn new(vertices: [usize; 3], all_points: &Vec<Vec3>) -> Self {
    let filtered_points = (0..all_points.len()).collect::<Vec<usize>>();

    Self::with_filtered_points(vertices, all_points, filtered_points)
  }

  pub fn with_filtered_points(
    vertices: [usize; 3],
    all_points: &Vec<Vec3>,
    filtered_points: Vec<usize>,
  ) -> Self {
    let ab = all_points[vertices[1]] - all_points[vertices[0]];
    let ac = all_points[vertices[2]] - all_points[vertices[0]];
    let cross = ab.cross(ac);

    let mut visible_points = Vec::new();
    let mut max_distant_point = HullPoint { point: usize::MAX, pseudo_volume: 0.0 };
    for p in filtered_points {
      if p != usize::MAX && (p == vertices[0] || p == vertices[1] || p == vertices[2]) {
        continue;
      }
      let dot = cross.dot(all_points[p] - all_points[vertices[0]]);

      if dot > max_distant_point.pseudo_volume {
        max_distant_point.point = p;
        max_distant_point.pseudo_volume = dot;
      }

      if dot > 0.0 {
        visible_points.push(p);
      }
    }

    let max_distant_point = if visible_points.is_empty() { None } else { Some(max_distant_point) };

    Self {
      vertices,
      visible_points,
      max_distant_point,
    }
  }
}


pub fn convex_hull(points: &Vec<Vec3>) -> (Vec<usize>, Vec<[usize; 3]>) {
  try_convex_hull(points).unwrap()
}


pub fn try_convex_hull(points: &Vec<Vec3>) -> Option<(Vec<usize>, Vec<[usize; 3]>)> {
  if points.len() < 3 {
    return None;
  }

  let mut normalized_points = points.to_vec();
  let _ = normalize(&mut normalized_points[..]);

  let mut hull_points = vec![0, 1, 2];
  let mut hull_faces = Vec::new();

  for (i, &p) in normalized_points.iter().enumerate() {
    if p.x < normalized_points[hull_points[0]].x {
      hull_points[0] = i;
    }

    if p.x > normalized_points[hull_points[1]].x {
      hull_points[1] = i;
    }

    if p.y > normalized_points[hull_points[2]].y {
      hull_points[2] = i;
    }
  }

  let face1 = HullFace::new([hull_points[0], hull_points[1], hull_points[2]], &normalized_points);
  let face2 = HullFace::new([hull_points[0], hull_points[2], hull_points[1]], &normalized_points);

  let mut stack = vec![face1, face2];
  while let Some(face) = stack.pop() {
    if face.max_distant_point.is_none() {
      hull_faces.push(face.vertices);
      continue;
    }

    let max_distant_point = face.max_distant_point.unwrap();
    hull_points.push(max_distant_point.point);

    let new_faces = [
      [face.vertices[0], face.vertices[1], max_distant_point.point],
      [face.vertices[1], face.vertices[2], max_distant_point.point],
      [face.vertices[2], face.vertices[0], max_distant_point.point],
    ];

    for new_face in new_faces {
      let new_face = HullFace::with_filtered_points(
        new_face,
        &normalized_points,
        face.visible_points.clone()
      );

      if new_face.visible_points.len() >= face.visible_points.len() {
        panic!("Invalid face");
      }
      stack.push(new_face);
    }
  }

  return Some((hull_points, hull_faces));
}


pub fn normalize(points: &mut [Vec3]) -> (Vec3, f32) {
  let mut aabb_min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
  let mut aabb_max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

  for c in points.iter() {
    aabb_min = aabb_min.min(*c);
    aabb_max = aabb_max.max(*c);
  }

  let diag = (aabb_max - aabb_min).length();
  let center = (aabb_max + aabb_min) / 2.0;

  for c in points.iter_mut() {
    *c = (*c - center) / diag;
  }

  (center, diag)
}
