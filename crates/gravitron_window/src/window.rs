use std::sync::mpsc::Sender;

use anyhow::Error;
use gravitron_plugin::config::window::WindowConfig;
use gravitron_utils::thread::Signal;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoop},
  platform::wayland::EventLoopBuilderExtWayland,
  window::{Window, WindowAttributes, WindowId},
};

pub struct WindowHandler {
  config: WindowConfig,
  ready_signal: Signal<Window>,
  sender: Sender<WindowEvent>,
}

impl WindowHandler {
  pub fn init(
    config: WindowConfig,
    ready_signal: Signal<Window>,
    sender: Sender<WindowEvent>,
  ) -> Result<(), Error> {
    let mut event_loop = EventLoop::builder();
    event_loop.with_any_thread(true);
    let event_loop = event_loop.build()?;

    event_loop.run_app(&mut WindowHandler {
      config,
      ready_signal,
      sender,
    })?;

    Ok(())
  }
}

impl ApplicationHandler for WindowHandler {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes = WindowAttributes::default()
      .with_title(self.config.title.clone())
      .with_inner_size(Size::Logical(LogicalSize::new(
        self.config.width as f64,
        self.config.height as f64,
      )));

    let window = event_loop
      .create_window(window_attributes)
      .expect("Error: Failed to create Window");

    self.ready_signal.send(window);
  }

  fn window_event(
    &mut self,
    _event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    self.sender.send(event).expect("Failed to send WindowEvent");
  }
}
