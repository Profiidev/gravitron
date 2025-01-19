use vulkan::VulkanConfig;

pub mod vulkan;
pub mod window;

#[derive(Default)]
pub struct AppConfig {
  pub window: window::WindowConfig,
  pub vulkan: VulkanConfig,
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
}
