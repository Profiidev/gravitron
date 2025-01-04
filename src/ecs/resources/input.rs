use std::collections::HashSet;

use winit::keyboard::KeyCode;

#[derive(Default, Clone)]
pub struct Input {
  inputs: HashSet<KeyCode>,
  cursor_pos: (f64, f64),
}

impl Input {
  pub fn is_key_pressed(&self, code: &KeyCode) -> bool {
    self.inputs.contains(code)
  }

  pub fn get_cursor_pos(&self) -> (f64, f64) {
    self.cursor_pos
  }

  pub fn get_cursor_x(&self) -> f64 {
    self.cursor_pos.0
  }

  pub fn get_cursor_y(&self) -> f64 {
    self.cursor_pos.1
  }

  pub fn press(&mut self, code: KeyCode) {
    self.inputs.insert(code);
  }

  pub fn release(&mut self, code: &KeyCode) {
    self.inputs.remove(code);
  }

  pub fn set_cursor_pos(&mut self, x: f64, y: f64) {
    self.cursor_pos = (x, y);
  }
}
