#[allow(dead_code)]
#[derive(Debug)]
pub struct EBO {
  pub id: u32,
}

impl Clone for EBO {
  fn clone(&self) -> Self { Self::new() }
}

#[allow(dead_code)]
impl EBO {
  pub fn new() -> Self {
    let mut id = 0;
    unsafe { gl::GenBuffers(1, &mut id); }

    return EBO { id };
  }

  pub fn clone(&self) -> Self {
    Self::new()
  }

  pub fn bind(&self) {
    unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id); }
  }

  pub fn delete(&self) {
    unsafe { gl::DeleteBuffers(1, &self.id); }
  }

  pub fn send_data(&self, data: &[u32]) {
    unsafe {
      gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        (data.len() * size_of::<u32>()) as isize,
        data.as_ptr().cast(),
        gl::STATIC_DRAW,
      );
    }
  }
}
