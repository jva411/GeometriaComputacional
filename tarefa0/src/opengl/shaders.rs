use std::fs::File;

use crate::opengl::shader::{Shader, ShaderType::{FRAGMENT, VERTEX}};

pub struct Shaders {
  pub vertex: Shader,
  pub fragment: Shader,
}

#[allow(dead_code)]
impl Shaders {
  pub fn new(vertex_source: String, fragment_source: String) -> Option<Self> {
    return Some(Self{
      vertex: Shader::new(VERTEX, vertex_source)?,
      fragment: Shader::new(FRAGMENT, fragment_source)?,
    });
  }

  pub fn from_files(vertex_file: &File, fragment_file: &File) -> Option<Self> {
    return Some(Self{
      vertex: Shader::from_file(VERTEX, vertex_file)?,
      fragment: Shader::from_file(FRAGMENT, fragment_file)?,
    });
  }

  pub fn delete(&self) {
    self.vertex.delete();
    self.fragment.delete();
  }
}
