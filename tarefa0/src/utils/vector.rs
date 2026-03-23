use glam::Vec3;

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
