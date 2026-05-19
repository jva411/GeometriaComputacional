mod lights;
mod objects;
mod opengl;
mod scene;
mod utils;


use crate::scene::window::Window;


const SCENE_WIDTH: u32 = 1280;
const UI_WIDTH: u32 = 350;
const WINDOW_HEIGHT: u32 = 850;
const WINDOW_WIDTH: u32 = SCENE_WIDTH + UI_WIDTH;


fn main() {
  let mut window = Window::new(
    "Computer Graphics",
    WINDOW_WIDTH,
    WINDOW_HEIGHT,
    SCENE_WIDTH,
  );

  window.init();
}
