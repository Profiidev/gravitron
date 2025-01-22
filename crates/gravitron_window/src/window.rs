use std::sync::mpsc::Sender;

use anyhow::Error;
use gravitron_plugin::config::window::WindowConfig;
use gravitron_utils::thread::Signal;
use log::debug;
#[cfg(target_os = "linux")]
use winit::platform::wayland::{ActiveEventLoopExtWayland, EventLoopBuilderExtWayland};
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoop},
  window::{Window, WindowAttributes, WindowId},
};

#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;

pub struct WindowHandler {
  config: WindowConfig,
  ready_signal: Signal<Window>,
  #[cfg(target_os = "linux")]
  wayland_signal: Signal<bool>,
  sender: Sender<WindowEvent>,
}

impl WindowHandler {
  pub fn init(
    config: WindowConfig,
    ready_signal: Signal<Window>,
    sender: Sender<WindowEvent>,
    #[cfg(target_os = "linux")] wayland_signal: Signal<bool>,
  ) -> Result<(), Error> {
    debug!("Creating EventLoop");
    let mut event_loop = EventLoop::builder();

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
      debug!("Configuring EventLoop");
      event_loop.with_any_thread(true);
    }

    debug!("Building EventLoop");
    let event_loop = event_loop.build()?;

    debug!("Running EventLoop");
    event_loop.run_app(&mut WindowHandler {
      config,
      ready_signal,
      sender,
      #[cfg(target_os = "linux")]
      wayland_signal,
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

    debug!("Creating Window");
    let window = event_loop
      .create_window(window_attributes)
      .expect("Error: Failed to create Window");

    self.ready_signal.send(window);
    #[cfg(target_os = "linux")]
    debug!("Window is using wayland: {}", event_loop.is_wayland());
    #[cfg(target_os = "linux")]
    self.wayland_signal.send(event_loop.is_wayland());
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
