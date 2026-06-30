use std::collections::HashMap;

use glam::{DVec3, Vec3};
use parry3d::math::Vec3 as PVec3;

pub fn calculate_normals(vertices: &Vec<Vec3>, indices: &Vec<u32>) -> Vec<Vec3> {
  let mut normals = vec![Vec3::ZERO; vertices.len()];

  for face in indices.chunks_exact(3) {
    let i0 = face[0] as usize;
    let i1 = face[1] as usize;
    let i2 = face[2] as usize;

    let v0 = vertices[i0];
    let v1 = vertices[i1];
    let v2 = vertices[i2];

    let n = (v1 - v0).cross(v2 - v0);
    normals[i0] += n;
    normals[i1] += n;
    normals[i2] += n;
  }

  for normal in normals.iter_mut() {
    *normal = normal.normalize_or_zero();
  }

  return normals;
}

pub fn pvec3_to_vec3(v: PVec3) -> Vec3 {
  return Vec3::new(v.x, v.y, v.z);
}
pub fn pvec3_vec_to_vec3_vec(v: &Vec<PVec3>) -> Vec<Vec3> {
  return v.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect();
}

pub fn vec3_to_pvec3(v: Vec3) -> PVec3 {
  return PVec3::new(v.x, v.y, v.z);
}
pub fn vec3_vec_to_pvec3_vec(v: &Vec<Vec3>) -> Vec<PVec3> {
  return v.iter().map(|p| PVec3::new(p.x, p.y, p.z)).collect();
}

pub fn wed_points(all_points: &Vec<Vec3>, candidates: &Vec<DVec3>, faces: &Vec<u32>) -> Vec<Vec3> {
  let all_points = all_points.iter().map(Vec3::as_dvec3).collect::<Vec<_>>();
  let avg_edge = average_edge_length(&all_points, faces);
  let min_dist = (avg_edge * 0.25).max(1e-5);

  let filtered_points = filter_close_points(&all_points, &candidates, min_dist);

  return filtered_points.iter().map(DVec3::as_vec3).collect::<Vec<_>>();
}


fn average_edge_length(vertices: &Vec<DVec3>, faces: &Vec<u32>) -> f64 {
  let mut total = 0.0f64;
  let mut count = 0u32;

  for chunk in faces.chunks_exact(3) {
    let a = vertices[chunk[0] as usize];
    let b = vertices[chunk[1] as usize];
    let c = vertices[chunk[2] as usize];
    total += (a - b).length() + (b - c).length() + (c - a).length();
    count += 3;
  }

  if count == 0 { 1.0 } else { total / count as f64 }
}

fn filter_close_points(existing: &Vec<DVec3>, candidates: &Vec<DVec3>, min_dist: f64) -> Vec<DVec3> {
  if min_dist <= 0.0 {
    return candidates.to_vec();
  }

  let cell_size = min_dist;
  let cell_of = |p: DVec3| -> (i64, i64, i64) {
    (
      (p.x / cell_size).floor() as i64,
      (p.y / cell_size).floor() as i64,
      (p.z / cell_size).floor() as i64,
    )
  };

  let mut grid: HashMap<(i64, i64, i64), Vec<DVec3>> = HashMap::new();
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
