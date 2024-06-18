pub mod app;
pub mod vulkan;
pub mod utils;

#[derive(Default)]
pub struct EngineConfig {
  pub app: app::AppConfig,
  pub vulkan: vulkan::VulkanConfig,
}

impl EngineConfig {
  pub fn set_app_config(mut self, app: app::AppConfig) -> Self {
    self.app = app;
    self
  }

  pub fn set_vulkan_config(mut self, vulkan: vulkan::VulkanConfig) -> Self {
    self.vulkan = vulkan;
    self
  }
}
