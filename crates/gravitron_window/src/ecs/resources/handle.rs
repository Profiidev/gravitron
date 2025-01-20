use winit::{
  raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle},
  window::Window,
};

pub struct WindowHandle {
  window: RawWindowHandle,
  display: RawDisplayHandle,
}

impl WindowHandle {
  pub fn window(&self) -> RawWindowHandle {
    self.window
  }

  pub fn display(&self) -> RawDisplayHandle {
    self.display
  }

  pub(crate) fn new(window: &Window) -> Option<Self> {
    Some(Self {
      window: window.window_handle().ok()?.as_raw(),
      display: window.display_handle().ok()?.as_raw(),
    })
  }
}
