use engine::EngineConfig;
use vulkan::VulkanConfig;

pub mod engine;
pub mod vulkan;
pub mod window;

#[derive(Default)]
pub struct AppConfig {
  pub window: window::WindowConfig,
  pub vulkan: VulkanConfig,
  pub engine: EngineConfig,
}

impl AppConfig {
  pub fn set_window_config(mut self, window: window::WindowConfig) -> Self {
    self.window = window;
    self
  }

  pub fn set_vulkan_config(mut self, vulkan: VulkanConfig) -> Self {
    self.vulkan = vulkan;
    self
  }

  pub fn set_engine_config(mut self, engine: EngineConfig) -> Self {
    self.engine = engine;
    self
  }
}
