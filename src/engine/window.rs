use anyhow::Error;
use winit::{
  application::ApplicationHandler, dpi::{LogicalSize, Size}, event_loop::EventLoop
};
#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;

use crate::{
  config::{app::AppConfig, vulkan::VulkanConfig}, util::signal::Signal, vulkan::Vulkan
};

pub struct Window {
  config: AppConfig,
  vulkan_config: VulkanConfig,
  instance: Option<Vulkan>,
  app_run: Signal,
  window_ready: Signal,
}

impl Window {
  //! Blocking
  pub fn init(
    config: AppConfig,
    vulkan_config: VulkanConfig,
    app_run: Signal,
    window_ready: Signal,
  ) -> Result<(), Error> {
    let event_loop = EventLoop::builder().with_any_thread(true).build()?;
    event_loop.run_app(&mut Window {
      config,
      vulkan_config,
      instance: None,
      app_run,
      window_ready,
    })?;

    Ok(())
  }
}

impl ApplicationHandler for Window {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title(self.config.title.clone())
      .with_inner_size(Size::Logical(LogicalSize::new(
        self.config.width as f64,
        self.config.height as f64,
      )));

    let window = event_loop.create_window(window_attributes).unwrap();

    let v = Vulkan::init(
      std::mem::take(&mut self.vulkan_config),
      &self.config,
      window,
    )
    .unwrap();
    self.instance = Some(v);

    self.window_ready.signal();
    self.app_run.wait();
  }

  fn window_event(
    &mut self,
    _event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        if let Some(mut v) = self.instance.take() {
          v.destroy();
        }
      }
      winit::event::WindowEvent::RedrawRequested => {
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
    if let Some(v) = &self.instance {
      v.request_redraw();
    }
  }
}
