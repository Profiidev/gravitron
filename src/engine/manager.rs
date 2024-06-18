use std::thread::{self, JoinHandle};

use crate::{config::EngineConfig, util::signal::Signal};

use super::window::Window;

pub trait Manager {
  fn init(config: EngineConfig) -> Self;
  fn run(self);
}

pub struct ClientManager {
  window_handle: JoinHandle<()>,
  app_run: Signal,
}

impl Manager for ClientManager {
  fn init(config: EngineConfig) -> Self {
    let window_ready = Signal::new();
    let app_run = Signal::new();

    let thread_window_ready = window_ready.clone();
    let thread_app_run = app_run.clone();

    let window_handle = thread::spawn(move || {
      Window::init(
        config.app,
        config.vulkan,
        thread_window_ready,
        thread_app_run,
      )
      .unwrap();
    });

    window_ready.wait();

    ClientManager {
      window_handle,
      app_run,
    }
  }

  fn run(self) {
    self.app_run.signal();

    self.window_handle.join().unwrap();
  }
}

pub struct ServerManager {}

impl Manager for ServerManager {
  fn init(config: EngineConfig) -> Self {
    println!("Server manager is initialized");
    ServerManager {}
  }

  fn run(self) {
    println!("Server manager is running");
  }
}
