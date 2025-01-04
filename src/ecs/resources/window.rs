use anyhow::Error;
use winit::{dpi::LogicalPosition, window::Window as WinitWindow};

pub use winit::window::CursorGrabMode;

#[derive(Default)]
pub struct Window {
  cursor_grab: Option<CursorGrabMode>,
  cursor_visible: Option<bool>,
  cursor_pos: Option<(f64, f64)>,
}

impl Window {
  pub fn set_cursor_grab(&mut self, mode: CursorGrabMode) {
    self.cursor_grab = Some(mode);
  }

  pub fn set_cursor_visible(&mut self, visible: bool) {
    self.cursor_visible = Some(visible);
  }

  pub fn set_cursor_pos(&mut self, x: f64, y: f64) {
    self.cursor_pos = Some((x, y));
  }

  pub fn execute(&mut self, window: WinitWindow) -> Result<(), Error> {
    if let Some(mode) = self.cursor_grab.take() {
      window.set_cursor_grab(mode)?;
    }
    if let Some(visible) = self.cursor_visible.take() {
      window.set_cursor_visible(visible);
    }
    if let Some((x, y)) = self.cursor_pos.take() {
      window.set_cursor_position(LogicalPosition::new(x, y))?;
    }

    Ok(())
  }
}
