use std::collections::HashSet;

use winit::{
  event::{ElementState, KeyEvent, WindowEvent},
  keyboard::{KeyCode, PhysicalKey},
};

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

  pub fn release(&mut self, code: &KeyCode) {
    self.inputs.remove(code);
  }

  pub(crate) fn handle_event(&mut self, event: &WindowEvent) {
    match event {
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(code),
            repeat: false,
            state,
            ..
          },
        ..
      } => match state {
        ElementState::Pressed => {
          self.inputs.insert(*code);
        }
        ElementState::Released => {
          self.inputs.remove(code);
        }
      },
      WindowEvent::CursorMoved { position, .. } => {
        self.cursor_pos = (position.x, position.y);
      }
      _ => (),
    }
  }
}
