
use gl::{AttachShader, CreateProgram, GetUniformLocation, LinkProgram, UseProgram};
use glam::Mat4;

use crate::opengl::shaders::Shaders;

#[allow(dead_code)]
pub struct Program {
  pub id: u32,
  pub shaders: Shaders,
}

#[allow(dead_code)]
impl Program {
  pub fn new(shaders: Shaders) -> Self {
    unsafe {
      let id = CreateProgram();

      AttachShader(id, shaders.vertex.id);
      AttachShader(id, shaders.fragment.id);
      LinkProgram(id);

      let mut success = 0;
      gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
      if success == 0 {
          let mut len = 0;
          gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
          let mut log_buffer: Vec<u8> = Vec::with_capacity(len.try_into().unwrap());
          gl::GetProgramInfoLog(id, len, &mut len, log_buffer.as_mut_ptr().cast());
          log_buffer.set_len(len.try_into().unwrap());
          let log = String::from_utf8(log_buffer).unwrap();
          println!("Falha ao linkar o Shader Program:\n\t{}", log);
      }

      shaders.delete();

      return Program { id, shaders };
    }
  }

  pub fn bind(&self) {
    unsafe {
      UseProgram(self.id);
    }
  }

  pub fn set_uniform_matrix4f(&self, name: &str, value: Mat4) -> Result<(), String> {
    let name = name.to_string() + "\0";
    unsafe {
      let location = GetUniformLocation(self.id, name.as_ptr().cast());
      if location == -1 {
        let mut n_uniforms = 0;
        gl::GetProgramiv(self.id, gl::ACTIVE_UNIFORMS, &mut n_uniforms);
        println!("n_uniforms: {}", n_uniforms);

        for i in 0..n_uniforms {
          const MAX_NAME_LENGTH: usize = 256;
          let mut name: [gl::types::GLchar; MAX_NAME_LENGTH] = [0; MAX_NAME_LENGTH];
          let mut len = 0;
          gl::GetActiveUniformName(
            self.id,
            i as gl::types::GLuint,
            MAX_NAME_LENGTH as gl::types::GLsizei,
            &mut len,
            name.as_mut_ptr().cast()
          );

          let name_u8 = name.iter().map(|c| *c as u8).collect::<Vec<u8>>();
          let name_str = String::from_utf8(name_u8).unwrap();
          println!("{}: {}", i, name_str);
        }

        return Err(format!("Uniform not found: {}", name));
      }

      gl::UniformMatrix4fv(
        location,
        1,
        gl::FALSE,
        value.to_cols_array().as_ptr()
      );

      return Ok(());
    };
  }

  pub fn set_uniform_vec3f<T: Into<[f32; 3]>>(&self, name: &str, value: T) -> Result<(), String> {
    let name = name.to_string() + "\0";
    unsafe {
      let location = GetUniformLocation(self.id, name.as_ptr().cast());
      if location == -1 {
        return Err(format!("Uniform not found: {}", name));
      }

      gl::Uniform3fv(location, 1, value.into().as_ptr());

      return Ok(());
    };
  }

  pub fn set_uniform1f(&self, name: &str, value: f32) -> Result<(), String> {
    let name = name.to_string() + "\0";
    unsafe {
      let location = GetUniformLocation(self.id, name.as_ptr().cast());
      if location == -1 {
        return Err(format!("Uniform not found: {}", name));
      }

      gl::Uniform1f(location, value);

      return Ok(());
    };
  }

  pub fn set_uniform1i(&self, name: &str, value: i32) -> Result<(), String> {
    let name = name.to_string() + "\0";
    unsafe {
      let location = GetUniformLocation(self.id, name.as_ptr().cast());
      if location == -1 {
        return Err(format!("Uniform not found: {}", name));
      }

      gl::Uniform1i(location, value);

      return Ok(());
    };
  }

  pub fn set_uniform_bool(&self, name: &str, value: bool) -> Result<(), String> {
    let name = name.to_string() + "\0";
    unsafe {
      let location = GetUniformLocation(self.id, name.as_ptr().cast());
      if location == -1 {
        return Err(format!("Uniform not found: {}", name));
      }

      gl::Uniform1i(location, value as i32);

      return Ok(());
    };
  }
}
