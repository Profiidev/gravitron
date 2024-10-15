use std::thread::JoinHandle;

use gravitron_ecs::ECS;
use gravitron_utils::thread::Signal;

use crate::vulkan::Vulkan;

pub struct EngineCommands {
  shutdown: bool,
  window_handle: Option<JoinHandle<()>>,
  shutdown_signal: Signal,
}

impl EngineCommands {
  pub fn create(window_handle: JoinHandle<()>, shutdown_signal: Signal) -> Self {
    Self {
      window_handle: Some(window_handle),
      shutdown: false,
      shutdown_signal,
    }
  }

  pub fn shutdown(&mut self) {
    self.shutdown = true;
  }

  pub fn execute(&mut self, ecs: &mut ECS) {
    if self.shutdown {
      let vulkan = ecs.get_resource_mut::<Vulkan>().unwrap();
      vulkan.destroy();

      self.shutdown_signal.signal();
      let handle = std::mem::take(&mut self.window_handle).unwrap();
      handle.join().unwrap();

      std::process::exit(0);
    }
  }
}
