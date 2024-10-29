use anyhow::Error;
use winit::{dpi::LogicalPosition, window::Window as WinitWindow};

pub use winit::window::CursorGrabMode;

pub struct Window {
  window_handle: WinitWindow,
}

impl Window {
  pub fn new(window: WinitWindow) -> Self {
    Self {
      window_handle: window,
    }
  }

  pub fn set_cursor_grab(&self, mode: CursorGrabMode) -> Result<(), Error> {
    self
      .window_handle
      .set_cursor_grab(mode)
      .map_err(|e| e.into())
  }

  pub fn set_cursor_visible(&self, visible: bool) {
    self.window_handle.set_cursor_visible(visible);
  }

  pub fn set_cursor_pos(&self, x: f64, y: f64) -> Result<(), Error> {
    self
      .window_handle
      .set_cursor_position(LogicalPosition::new(x, y))
      .map_err(|e| e.into())
  }
}
