#[allow(dead_code)]
#[derive(Debug)]
pub struct VBO {
  pub id: u32,
}

impl Clone for VBO {
  fn clone(&self) -> Self { Self::new() }
}

#[allow(dead_code)]
impl VBO {
  pub fn new() -> Self {
    let mut id = 0;
    unsafe { gl::GenBuffers(1, &mut id); }

    return VBO { id };
  }

  pub fn clone(&self) -> Self {
    Self::new()
  }

  pub fn bind(&self) {
    unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.id); }
  }

  pub fn delete(&self) {
    unsafe { gl::DeleteBuffers(1, &self.id); }
  }

  pub fn send_data(&self, data: &[f32]) {
    unsafe {
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (data.len() * size_of::<f32>()) as isize,
        data.as_ptr().cast(),
        gl::STATIC_DRAW,
      );
    }
  }

  pub fn update_data(&self, offset: u32, data: &[f32]) {
    unsafe {
      gl::BufferSubData(
        gl::ARRAY_BUFFER,
        offset as isize,
        (data.len() * size_of::<f32>()) as isize,
        data.as_ptr().cast(),
      );
    }
  }
}
