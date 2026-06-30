use glam::DVec3;
use rand::RngExt;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OctreeNodeType {
  IN,
  OUT,
  PARTIAL,
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
  pub min: DVec3,
  pub max: DVec3,
}

#[derive(Debug, Clone)]
pub struct OctreeNode {
  pub aabb: AABB,
  pub node_type: OctreeNodeType,
  pub children: Option<Vec<Box<OctreeNode>>>,
}

impl OctreeNode {
  fn new(aabb: AABB, node_type: OctreeNodeType) -> Self {
    OctreeNode {
      aabb,
      node_type,
      children: None,
    }
  }

  pub fn get_leaves_centroids(&self, points: &Vec<DVec3>, faces: &Vec<u32>) -> Vec<DVec3> {
    let mut new_points = Vec::new();
    self.get_leaves_centroids_rec(points, faces, &mut new_points);
    new_points
  }

  fn get_leaves_centroids_rec(&self, points: &Vec<DVec3>, faces: &Vec<u32>, new_points: &mut Vec<DVec3>) {
    if self.node_type == OctreeNodeType::OUT {
      return;
    }

    if self.node_type == OctreeNodeType::IN {
      let mut rng = rand::rng();
      let cell_diagonal = (self.aabb.max - self.aabb.min).length();
      let jitter_scale = cell_diagonal * 1e-3;
      let point = ((self.aabb.min + self.aabb.max) * 0.5) + DVec3::new(rng.random(), rng.random(), rng.random()) * jitter_scale;
      new_points.push(point);
      return;
    }

    if let Some(children) = &self.children {
      for child in children.iter() {
        child.get_leaves_centroids_rec(points, faces, new_points);
      }
    }
  }

  pub fn generate_from_mesh(points: &Vec<DVec3>, faces: &Vec<u32>, max_depth: u32) -> Self {
    let mut min = DVec3::splat(f64::MAX);
    let mut max = DVec3::splat(f64::MIN);

    for &point in points.iter() {
      min = min.min(point) * 0.9999999;
      max = max.max(point) * 0.9999999;
    }

    let root_aabb = AABB { min, max };
    let min = root_aabb.min.min_element();
    let max = root_aabb.max.max_element();

    let root_aabb = AABB { min: DVec3::splat(min), max: DVec3::splat(max) };
    let mut root_node = OctreeNode::new(root_aabb, OctreeNodeType::PARTIAL);
    OctreeNode::subdivide_node(points, faces, &mut root_node, 0, max_depth);

    root_node
  }

  fn subdivide_node(points: &Vec<DVec3>, faces: &Vec<u32>, node: &mut OctreeNode, depth: u32, max_depth: u32) {
    if depth >= max_depth {
      return;
    }

    let mut children = Vec::new();
    let mid = (node.aabb.min + node.aabb.max) * 0.5;

    for i in 0..8 {
      let min = DVec3::new(
        if (i & 1) == 0 { node.aabb.min.x } else { mid.x },
        if (i & 2) == 0 { node.aabb.min.y } else { mid.y },
        if (i & 4) == 0 { node.aabb.min.z } else { mid.z },
      );
      let max = DVec3::new(
        if (i & 1) == 0 { mid.x } else { node.aabb.max.x },
        if (i & 2) == 0 { mid.y } else { node.aabb.max.y },
        if (i & 4) == 0 { mid.z } else { node.aabb.max.z },
      );

      let child_aabb = AABB { min, max };
      let child_node_type = mesh_classify_aabb(points, faces, &child_aabb);
      let mut child_node = OctreeNode::new(child_aabb, child_node_type);

      match child_node.node_type {
        OctreeNodeType::PARTIAL => OctreeNode::subdivide_node(points, faces, &mut child_node, depth + 1, max_depth),
        _ => { }
      }
      children.push(Box::new(child_node));
    }

    node.children = Some(children);
  }
}

fn mesh_classify_aabb(points: &Vec<DVec3>, faces: &Vec<u32>, aabb: &AABB) -> OctreeNodeType {
  let center = (aabb.min + aabb.max) / 2.0;
  let half_size = (aabb.max - aabb.min) / 2.0;

  for i in (0..faces.len()).step_by(3) {
    let v0 = points[faces[i] as usize];
    let v1 = points[faces[i + 1] as usize];
    let v2 = points[faces[i + 2] as usize];

    if triangle_overlaps_box(center, half_size, v0, v1, v2) {
      return OctreeNodeType::PARTIAL;
    }
  }

  mesh_classify_point(points, faces, center)
}

fn mesh_classify_point(points: &Vec<DVec3>, faces: &Vec<u32>, point: DVec3) -> OctreeNodeType {
  let directions = [
    DVec3::new(0.3123, 0.7854, 0.5352).normalize(),
    DVec3::new(-0.6121, 0.1857, -0.7358).normalize(),
    DVec3::new(0.8124, -0.5852, 0.1359).normalize(),
  ];

  let mut in_count = 0;

  for dir in directions {
    let mut intersections = 0;
    for i in (0..faces.len()).step_by(3) {
      let v0 = points[faces[i] as usize];
      let v1 = points[faces[i + 1] as usize];
      let v2 = points[faces[i + 2] as usize];

      if ray_intersects_triangle(point, dir, v0, v1, v2) {
        intersections += 1;
      }
    }

    if intersections % 2 == 1 {
      in_count += 1;
    }
  }

  if in_count >= 2 {
    OctreeNodeType::IN
  } else {
    OctreeNodeType::OUT
  }
}

fn triangle_overlaps_box(boxcenter: DVec3, boxhalfsize: DVec3, trivet0: DVec3, trivet1: DVec3, trivet2: DVec3) -> bool {
  let v0 = trivet0 - boxcenter;
  let v1 = trivet1 - boxcenter;
  let v2 = trivet2 - boxcenter;

  let e0 = v1 - v0;
  let e1 = v2 - v1;
  let e2 = v0 - v2;

  let axes = [
    DVec3::X.cross(e0), DVec3::X.cross(e1), DVec3::X.cross(e2),
    DVec3::Y.cross(e0), DVec3::Y.cross(e1), DVec3::Y.cross(e2),
    DVec3::Z.cross(e0), DVec3::Z.cross(e1), DVec3::Z.cross(e2),
  ];

  for axis in axes {
    let p0 = v0.dot(axis);
    let p1 = v1.dot(axis);
    let p2 = v2.dot(axis);
    let r = boxhalfsize.x * axis.x.abs() + boxhalfsize.y * axis.y.abs() + boxhalfsize.z * axis.z.abs();
    if p0.max(p1.max(p2)) < -r || p0.min(p1.min(p2)) > r {
      return false;
    }
  }

  let box_axes = [DVec3::X, DVec3::Y, DVec3::Z];
  for &axis in &box_axes {
    let p0 = v0.dot(axis);
    let p1 = v1.dot(axis);
    let p2 = v2.dot(axis);
    let r = boxhalfsize.dot(axis.abs());
    if p0.max(p1.max(p2)) < -r || p0.min(p1.min(p2)) > r {
      return false;
    }
  }

  let normal = e0.cross(e1);
  let p0 = v0.dot(normal);
  let r = boxhalfsize.dot(normal.abs());
  if p0.abs() > r {
    return false;
  }

  true
}

fn ray_intersects_triangle(orig: DVec3, dir: DVec3, v0: DVec3, v1: DVec3, v2: DVec3) -> bool {
  const EPSILON: f64 = 0.000001;
  let edge1 = v1 - v0;
  let edge2 = v2 - v0;
  let h = dir.cross(edge2);
  let a = edge1.dot(h);
  if a > -EPSILON && a < EPSILON {
    return false;
  }

  let f = 1.0 / a;
  let s = orig - v0;
  let u = f * s.dot(h);
  if u < 0.0 || u > 1.0 {
    return false;
  }

  let q = s.cross(edge1);
  let v = f * dir.dot(q);
  if v < 0.0 || u + v > 1.0 {
    return false;
  }

  let t = f * edge2.dot(q);
  if t < EPSILON {
    return false;
  }

  return true;
}

pub struct PointOctree {
  pub aabb: AABB,
  pub indices: Vec<usize>,
  pub children: Option<Vec<Box<PointOctree>>>,
}

impl PointOctree {
  /// Constrói a Octree a partir de uma lista de pontos e seus índices
  pub fn new(points: &Vec<DVec3>, indices: Vec<usize>, aabb: AABB, depth: u32, max_depth: u32) -> Self {
    let max_points_per_leaf = 16;

    // Condição de parada: limite de profundidade ou poucos pontos no nó
    if depth >= max_depth || indices.len() <= max_points_per_leaf {
      return Self { aabb, indices, children: None };
    }

    let mid = (aabb.min + aabb.max) * 0.5;
    let mut children_indices = vec![Vec::new(); 8];

    // Distribui os pontos entre os 8 octantes
    for &idx in &indices {
      let p = points[idx];
      let mut octant = 0;
      if p.x >= mid.x { octant |= 1; }
      if p.y >= mid.y { octant |= 2; }
      if p.z >= mid.z { octant |= 4; }
      children_indices[octant].push(idx);
    }

    // Cria os nós filhos
    let mut children = Vec::with_capacity(8);
    for i in 0..8 {
      let min = DVec3::new(
        if (i & 1) == 0 { aabb.min.x } else { mid.x },
        if (i & 2) == 0 { aabb.min.y } else { mid.y },
        if (i & 4) == 0 { aabb.min.z } else { mid.z },
      );
      let max = DVec3::new(
        if (i & 1) == 0 { mid.x } else { aabb.max.x },
        if (i & 2) == 0 { mid.y } else { aabb.max.y },
        if (i & 4) == 0 { mid.z } else { aabb.max.z },
      );

      children.push(Box::new(PointOctree::new(
        points,
        children_indices[i].clone(),
        AABB { min, max },
        depth + 1,
        max_depth
      )));
    }

    Self {
      aabb,
      indices: Vec::new(),
      children: Some(children),
    }
  }

  pub fn query_sphere(&self, points: &Vec<DVec3>, center: DVec3, radius_sq: f64, result: &mut Vec<usize>) {
    if !self.aabb_intersects_sphere(center, radius_sq) {
      return;
    }

    if let Some(children) = &self.children {
      for child in children {
        child.query_sphere(points, center, radius_sq, result);
      }
    } else {
      for &idx in &self.indices {
        let dist_sq = (points[idx] - center).length_squared();
        if dist_sq <= radius_sq {
          result.push(idx);
        }
      }
    }
  }

  fn aabb_intersects_sphere(&self, center: DVec3, radius_sq: f64) -> bool {
    let x = center.x.max(self.aabb.min.x).min(self.aabb.max.x);
    let y = center.y.max(self.aabb.min.y).min(self.aabb.max.y);
    let z = center.z.max(self.aabb.min.z).min(self.aabb.max.z);

    let closest_point = DVec3::new(x, y, z);
    (closest_point - center).length_squared() <= radius_sq
  }
}
