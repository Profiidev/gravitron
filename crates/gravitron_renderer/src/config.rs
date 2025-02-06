use ash::vk;

#[derive(Default, Clone)]
pub struct VulkanConfig {
  pub renderer: RendererConfig<'static>,
}

impl VulkanConfig {
  pub fn set_renderer_config(mut self, renderer: RendererConfig<'static>) -> Self {
    self.renderer = renderer;
    self
  }
}

#[derive(Default, Clone)]
pub struct RendererConfig<'a> {
  pub layers: Vec<&'a std::ffi::CStr>,
  pub instance_extensions: Vec<&'a std::ffi::CStr>,
  pub device_extensions: Vec<&'a std::ffi::CStr>,
  pub device_features: vk::PhysicalDeviceFeatures,
}

impl<'a> RendererConfig<'a> {
  pub fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }
}
