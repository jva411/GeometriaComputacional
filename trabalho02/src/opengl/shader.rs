use std::fs::File;

#[allow(non_snake_case)]
pub mod ShaderType {
  #[derive(Debug, Clone, Copy)]
  pub struct Type {
    pub gl_type: u32,
  }

  pub const VERTEX: Type = Type { gl_type: gl::VERTEX_SHADER };
  pub const FRAGMENT: Type = Type { gl_type: gl::FRAGMENT_SHADER };
}

#[allow(dead_code)]
pub struct Shader {
  pub id: u32,
  pub _type: ShaderType::Type,
  pub source: String,
}

impl Shader {
  pub fn new(_type: ShaderType::Type, source: String) -> Option<Self> {
    unsafe {
      let id = gl::CreateShader(_type.gl_type);
      if id == 0 {
        println!("Shader::new -> Failed to create shader, id == 0");
        return None;
      }

      gl::ShaderSource(
        id,
        1,
        &(source.as_bytes().as_ptr().cast()),
        &(source.len().try_into().unwrap()),
      );
      gl::CompileShader(id);

      let mut success = 0;
      gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
      if success == 0 {
        let mut len = 0;
        gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        let mut log_buffer: Vec<u8> = Vec::with_capacity(len.try_into().unwrap());
        gl::GetShaderInfoLog(id, len, &mut len, log_buffer.as_mut_ptr().cast());
        log_buffer.set_len(len.try_into().unwrap());
        let log = String::from_utf8(log_buffer).unwrap();
        println!("Failed to compile shader:\n\t{}", log);
        return None;
      }

      return Some(Shader { id, _type, source });
    }
  }

  pub fn from_file(_type: ShaderType::Type, file: &File) -> Option<Self> {
    let source = std::io::read_to_string(file).unwrap();
    return Shader::new(_type, source);
  }

  pub fn delete(&self) {
    unsafe {
      gl::DeleteShader(self.id);
    }
  }
}
