use gravitron_renderer::config::VulkanConfig;

pub mod app;

#[derive(Default)]
pub struct EngineConfig {
  pub app: app::AppConfig,
  pub vulkan: VulkanConfig,
}

impl EngineConfig {
  pub fn set_app_config(mut self, app: app::AppConfig) -> Self {
    self.app = app;
    self
  }

  pub fn set_vulkan_config(mut self, vulkan: VulkanConfig) -> Self {
    self.vulkan = vulkan;
    self
  }
}
