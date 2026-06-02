use glam::{Vec3};
use uuid::Uuid;

use crate::{implement_partial_Object, implement_transformable, objects::object::{Object, ObjectType}, opengl::{ebo::EBO, renderer::Renderer, vao::VAO, vbo::VBO}, utils::{core::SIZE_F32, material::Material, ray::Ray, transform::Transform}};

// struct GlobalProps {
//   origem: Vec3,
//   normal: Vec3,
//   up: Vec3,
//   right: Vec3,
// }

pub struct Square {
  pub id: Uuid,
  pub name: String,

  pub origem: Vec3,
  pub normal: Vec3,

  pub transform: Transform,
  pub material: Material,

  pub vao: VAO,
  pub vbo: VBO,
  pub ebo: EBO,

  // global_props: GlobalProps,
}

pub const VERTICES: [f32; (3 + 3) * 4] = [
  // X     Y    Z     Nx    Ny    Nz
  -0.5, -0.5,  0.0,   0.0,  0.0, 1.0,
   0.5, -0.5,  0.0,   0.0,  0.0, 1.0,
   0.5,  0.5,  0.0,   0.0,  0.0, 1.0,
  -0.5,  0.5,  0.0,   0.0,  0.0, 1.0,
];

pub const INDICES: [u32; 3 * 2 * 1] = [
  0, 1, 2,  0, 2, 3
];

pub const STRIDE: u32 = (3+3) * SIZE_F32;
pub const SKIPS: [u32; 2] = [0, 3 * SIZE_F32];

impl Square {
  pub fn new(name: String, origem: Vec3, normal: Vec3) -> Self {
    let vao = VAO::new();
    let vbo = VBO::new();
    let ebo = EBO::new();
    vao.bind();
    vbo.bind();
    ebo.bind();

    for i in 0..SKIPS.len() {
      vao.add_attribute(i as u32, STRIDE, SKIPS[i]);
    }

    vbo.send_data(&VERTICES);
    ebo.send_data(&INDICES);

    return Self {
      id: Uuid::new_v4(),
      name: name,
      origem,
      normal,
      transform: Transform::default(),
      material: Material::default(),
      vao,
      vbo,
      ebo,

      // global_props: GlobalProps {
      //   origem,
      //   normal,
      //   up: Vec3::Y,
      //   right: Vec3::X,
      // },
    };
  }

  // pub fn edge_global_intersecton(&self, u: Vec3, v: Vec3) -> Option<f32> {
  //   let uv = v - u;
  //   let d = uv.dot(self.global_props.normal);

  //   if d.abs() < 1e-6 {
  //     return None;
  //   }

  //   let ou = self.global_props.origem - u;
  //   let t = ou.dot(self.global_props.normal) / d;
  //   if t < 0.0 || t > 1.0 {
  //     return None;
  //   }

  //   let hit_point_local = u + uv * t;
  //   let w = hit_point_local - self.global_props.origem;

  //   let local_x = w.dot(self.global_props.right) / self.global_props.right.length_squared();
  //   let local_y = w.dot(self.global_props.up) / self.global_props.up.length_squared();

  //   if local_x < -0.5 || local_x > 0.5 || local_y < -0.5 || local_y > 0.5 {
  //     return None;
  //   }

  //   return Some(t);
  // }
}


impl Object for Square {
  implement_partial_Object!();

  fn get_type(&self) -> ObjectType { ObjectType::Square }

  fn tick(&mut self) {
  //   let model = self.transform.build_model();
  //   self.global_props.origem = model.mul(self.origem.extend(1.0)).truncate();
  //   self.global_props.normal = model.mul(self.normal.extend(0.0)).truncate();
  //   self.global_props.right = model.mul(Vec4::X).truncate();
  //   self.global_props.up = model.mul(Vec4::Y).truncate();
  }

  fn draw(&self, renderer:  &mut Renderer, base_transform: Option<Transform>) {
    let program = &renderer.current_program;

    self.vao.bind();
    self.vbo.bind();
    self.ebo.bind();

    let model_transform = match base_transform {
      Some(t) => &self.transform.concat(&t),
      None => &self.transform,
    };
    let model_transform = model_transform.clone();
    model_transform.send_to_program(&program);
    self.material.send_to_program(&program);

    unsafe {
      gl::DrawElements(gl::TRIANGLES, INDICES.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
    }
  }

  fn clone(&self) -> Self {
    let mut plane = Square::new(self.name.clone(), self.origem, self.normal);
    plane.transform = self.transform.clone();
    plane.material = self.material.clone();
    return plane;
  }

  fn ray_intersection(&self, ray: Ray) -> Option<f32> {
    let d = ray.direction.dot(self.normal);

    if d.abs() < 1e-6 {
      return None;
    }

    let p0r0 = self.origem - ray.origin;
    let t = p0r0.dot(self.normal) / d;

    if t < 0.0 {
      return None;
    }

    let hit_point_local = ray.origin + t * ray.direction;

    if   hit_point_local.x < -0.5
      || hit_point_local.x >  0.5
      || hit_point_local.y < -0.5
      || hit_point_local.y >  0.5
    {
      return None;
    }

    return Some(t);
  }
}

implement_transformable!(Square);
