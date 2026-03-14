use std::{fs::File, rc::Rc};

use crate::opengl::{program::Program, shaders::Shaders};

#[allow(dead_code)]
pub enum ProgramType {
  Common,
  Light,
  Grid,
}

#[allow(dead_code)]
pub struct Renderer {
  pub programs: [Rc<Program>; 3],
  pub current_program: Rc<Program>,
}

#[allow(dead_code)]
impl Renderer {
  pub fn new() -> Option<Self> {
    let triangle_vertex_shader_file = File::open("assets/shaders/triangle/vertex.glsl").expect("Failed to open triangle vertex shader file");
    let triangle_fragment_shader_file = File::open("assets/shaders/triangle/fragment.glsl").expect("Failed to open triangle fragment shader file");
    let shaders = Shaders::from_files(&triangle_vertex_shader_file, &triangle_fragment_shader_file)?;
    let program = Program::new(shaders);

    let light_vertex_shader_file = File::open("assets/shaders/light/vertex.glsl").expect("Failed to open light vertex shader file");
    let light_fragment_shader_file = File::open("assets/shaders/light/fragment.glsl").expect("Failed to open light fragment shader file");
    let light_shaders = Shaders::from_files(&light_vertex_shader_file, &light_fragment_shader_file)?;
    let light_program = Program::new(light_shaders);

    let grid_vertex_shader_file = File::open("assets/shaders/grid/vertex.glsl").expect("Failed to open grid vertex shader file");
    let grid_fragment_shader_file = File::open("assets/shaders/grid/fragment.glsl").expect("Failed to open grid fragment shader file");
    let grid_shaders = Shaders::from_files(&grid_vertex_shader_file, &grid_fragment_shader_file)?;
    let grid_program = Program::new(grid_shaders);

    let current_program = Rc::new(program);
    return Some(Renderer {
      programs: [current_program.clone(), Rc::new(light_program), Rc::new(grid_program)],
      current_program,
    });
  }

  pub fn bind_program(&mut self, program_type: ProgramType) {
    let program_ref = &self.programs[program_type as usize];
    program_ref.bind();
    self.current_program = program_ref.clone();
  }

  pub fn clear(&self, width: u32, height: u32) {
    unsafe {
      gl::Viewport(0, 0, width as i32, height as i32);
      gl::Disable(gl::SCISSOR_TEST);

      gl::ClearColor(0.53, 0.81, 0.92, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

      gl::Enable(gl::DEPTH_TEST);
      gl::Enable(gl::CULL_FACE);
    }
  }
}
