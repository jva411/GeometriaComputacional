use glam::Vec3;
use uuid::Uuid;
use crate::geometry::tetrahedralization::Tetrahedron;
use crate::opengl::ebo::EBO;
use crate::opengl::vao::VAO;
use crate::opengl::vbo::VBO;
use crate::utils::core::SIZE_F32;
use crate::{implement_partial_Object, implement_transformable};
use crate::objects::object::{Object, ObjectType};
use crate::opengl::renderer::Renderer;
use crate::utils::transform::Transform;
use crate::utils::material::Material;

#[derive(Clone)]
pub struct TetrahedronPart {
  pub points: Vec<Vec3>,
  pub tetrahedrons: Vec<Tetrahedron>,
}

pub struct TetrahedronObject {
  pub id: Uuid,
  pub name: String,

  pub transform: Transform,
  pub material: Material,

  pub parts: Vec<TetrahedronPart>,

  pub render_mesh: bool,
  pub render_wireframe: bool,
  pub render_points: bool,
  pub last_shrink: f32,
  pub shrink: f32,
  vao: VAO,
  vbo: VBO,
  ebo: EBO,
  elements_count: i32,
}

impl TetrahedronObject {
  pub fn new(
    name: String,
    parts: Vec<TetrahedronPart>,
  ) -> Self {
    println!("Tetra Parts: {}", parts.len());
    Self {
      id: Uuid::new_v4(),
      name,
      transform: Transform::default(),
      material: Material::default(),
      parts,
      render_mesh: true,
      render_wireframe: false,
      render_points: false,
      last_shrink: f32::MIN,
      shrink: 0.0,
      vao: VAO::new(),
      vbo: VBO::new(),
      ebo: EBO::new(),
      elements_count: 0,
    }
  }

  pub fn build_render_buffers(&mut self) {
    let mut vertex_data: Vec<f32> = Vec::new();
    let mut index_data: Vec<u32> = Vec::new();
    let mut current_idx = 0;

    for part in &self.parts {
      for tetra in &part.tetrahedrons {
        let p0 = part.points[tetra.0 as usize];
        let p1 = part.points[tetra.1 as usize];
        let p2 = part.points[tetra.2 as usize];
        let p3 = part.points[tetra.3 as usize];

        let centroid = (p0 + p1 + p2 + p3) * 0.25;

        let faces = [
          (p0, p1, p2),
          (p0, p2, p3),
          (p0, p3, p1),
          (p1, p3, p2),
        ];

        for face in faces {
          let mut v0 = face.0;
          let mut v1 = face.1;
          let mut v2 = face.2;

          if self.shrink > 0.0 {
            v0 = v0.lerp(centroid, self.shrink);
            v1 = v1.lerp(centroid, self.shrink);
            v2 = v2.lerp(centroid, self.shrink);
          }

          let edge1 = v1 - v0;
          let edge2 = v2 - v0;
          let mut normal = edge1.cross(edge2).normalize();

          if normal.dot(v0 - centroid) < 0.0 {
            normal = -normal;
            std::mem::swap(&mut v1, &mut v2);
          }

          vertex_data.extend_from_slice(&[v0.x, v0.y, v0.z, normal.x, normal.y, normal.z]);
          vertex_data.extend_from_slice(&[v1.x, v1.y, v1.z, normal.x, normal.y, normal.z]);
          vertex_data.extend_from_slice(&[v2.x, v2.y, v2.z, normal.x, normal.y, normal.z]);

          index_data.extend_from_slice(&[current_idx, current_idx + 1, current_idx + 2]);
          current_idx += 3;
        }
      }
    }

    self.vao.bind();
    self.vbo.bind();
    self.ebo.bind();

    let stride = 6 * SIZE_F32;
    self.vao.add_attribute(0, stride, 0);
    self.vao.add_attribute(1, stride, (3 * SIZE_F32) as u32);

    self.vbo.send_data(&vertex_data);
    self.ebo.send_data(&index_data);

    self.elements_count = index_data.len() as i32;
  }
}

impl Object for TetrahedronObject {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Tetrahedron }

  fn tick(&mut self) {
    if self.last_shrink != self.shrink {
      self.last_shrink = self.shrink;
      self.build_render_buffers();
    }
  }

  fn draw(&self, renderer: &mut Renderer, base_transform: Option<Transform>) {
    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };

    let program = &renderer.current_program;
    self.vao.bind();
    model_transform.send_to_program(program);

    if self.render_mesh {
      unsafe {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        gl::DrawElements(gl::TRIANGLES, self.elements_count, gl::UNSIGNED_INT, std::ptr::null());
      }
    }

    if self.render_wireframe {
      program.set_uniform_bool("uSimplex", true).unwrap();
      program.set_uniform_bool("uUseSimplexColor", true).unwrap();
      program.set_uniform_vec3f("uSimplexColor", Vec3::new(1.0, 1.0, 1.0)).unwrap();
      unsafe {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::LineWidth(1.5);
        gl::DrawElements(gl::TRIANGLES, self.elements_count, gl::UNSIGNED_INT, std::ptr::null());
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
      }
      program.set_uniform_bool("uUseSimplexColor", false).unwrap();
      program.set_uniform_bool("uSimplex", false).unwrap();
    }

    if self.render_points {
      program.set_uniform_bool("uSimplex", true).unwrap();
      program.set_uniform_bool("uUseSimplexColor", true).unwrap();
      program.set_uniform_vec3f("uSimplexColor", Vec3::new(0.0, 0.0, 0.0)).unwrap();
      unsafe {
        gl::PointSize(5.0);
        gl::DrawArrays(gl::POINTS, 0, self.elements_count / 3);
      }
      program.set_uniform_bool("uUseSimplexColor", false).unwrap();
      program.set_uniform_bool("uSimplex", false).unwrap();
    }
  }

  fn clone(&self) -> Self {
    let mut obj = TetrahedronObject::new(self.name.clone(), self.parts.clone());
    obj.transform = self.transform.clone();
    obj.material = self.material.clone();
    obj.shrink = self.shrink;
    obj.last_shrink = self.last_shrink;
    obj.elements_count = self.elements_count;
    obj
  }
}

implement_transformable!(TetrahedronObject);
