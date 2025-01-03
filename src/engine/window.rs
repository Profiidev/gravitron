use std::sync::mpsc::Sender;

use anyhow::Error;
use gravitron_utils::thread::Signal;
use log::{debug, info};
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
  event::{ElementState, KeyEvent},
  event_loop::EventLoop,
  keyboard::PhysicalKey,
  window::Window as WinitWindow,
};
#[cfg(target_os = "linux")]
use winit::{
  event_loop::EventLoopBuilder,
  platform::wayland::{ActiveEventLoopExtWayland, EventLoopBuilderExtWayland},
};
#[cfg(target_os = "windows")]
use winit::{event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows};

use crate::{config::EngineConfig, vulkan::Vulkan};

use super::WindowMessage;

pub struct Window {
  config: EngineConfig,
  app_run: Signal,
  window_ready: Signal<(Vulkan, WinitWindow)>,
  shutdown: Signal,
  send: Sender<WindowMessage>,
}

impl Window {
  //! Blocking
  pub fn init(
    config: EngineConfig,
    app_run: Signal,
    window_ready: Signal<(Vulkan, WinitWindow)>,
    shutdown: Signal,
    send: Sender<WindowMessage>,
  ) -> Result<(), Error> {
    let mut event_loop = EventLoop::builder();

    #[cfg(target_os = "linux")]
    let event_loop =
      <EventLoopBuilder<()> as EventLoopBuilderExtWayland>::with_any_thread(&mut event_loop, true);
    #[cfg(target_os = "windows")]
    let event_loop =
      <EventLoopBuilder<()> as EventLoopBuilderExtWindows>::with_any_thread(&mut event_loop, true);

    let event_loop = event_loop.build()?;

    debug!("Starting Event Loop");
    event_loop.run_app(&mut Window {
      config,
      app_run,
      window_ready,
      shutdown,
      send,
    })?;

    Ok(())
  }
}

impl ApplicationHandler for Window {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title(self.config.app.title.clone())
      .with_inner_size(Size::Logical(LogicalSize::new(
        self.config.app.width as f64,
        self.config.app.height as f64,
      )));

    debug!("Creating Window");
    let window = event_loop.create_window(window_attributes).unwrap();

    debug!("Creating Vulkan Instnace");
    let v = Vulkan::init(
      std::mem::take(&mut self.config.vulkan),
      &self.config.app,
      &window,
      #[cfg(target_os = "linux")]
      event_loop.is_wayland(),
    )
    .unwrap();

    self.window_ready.send((v, window));
    debug!("Waiting for Engine start");
    self.app_run.wait();
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        info!("Window sending exit request");
        event_loop.exit();
        self.send.send(WindowMessage::Exit).unwrap();
      }
      winit::event::WindowEvent::RedrawRequested => {
        debug!("Redraw Request");
      }
      winit::event::WindowEvent::KeyboardInput {
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
          self.send.send(WindowMessage::KeyPressed(code)).unwrap();
        }
        ElementState::Released => {
          self.send.send(WindowMessage::KeyReleased(code)).unwrap();
        }
      },
      winit::event::WindowEvent::CursorMoved { position, .. } => {
        self
          .send
          .send(WindowMessage::MouseMove(position.x, position.y))
          .unwrap();
      }
      _ => {}
    }
  }

  fn new_events(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _cause: winit::event::StartCause,
  ) {
    if self.shutdown.is_signaled() {
      event_loop.exit();
    }
  }
}
