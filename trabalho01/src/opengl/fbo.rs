#[allow(dead_code)]
#[derive(Debug)]
pub struct FBO {
  pub id: u32,
  pub texture_id: u32,
  pub rbo_id: u32,
}

#[allow(dead_code)]
impl FBO {
  pub fn new() -> FBO {
    let mut fbo = FBO {
      id: 0,
      texture_id: 0,
      rbo_id: 0,
    };

    unsafe {
      gl::GenFramebuffers(1, &mut fbo.id);
      gl::GenTextures(1, &mut fbo.texture_id);
      gl::GenRenderbuffers(1, &mut fbo.rbo_id);
    }

    return fbo;
  }

  pub fn bind(&self) {
    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
      gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB.cast_signed(), 800, 600, 0, gl::RGB, gl::UNSIGNED_BYTE, 0 as *const _);

      gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
      gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.texture_id, 0);

      gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo_id);
      gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, 800, 600);
      gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, self.rbo_id);

      if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
        panic!("Framebuffer not complete!");
      }
    }
  }

  pub fn unbind(&self) {
    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
      gl::BindTexture(gl::TEXTURE_2D, 0);
      gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
    }
  }
}
