use ash::vk;

use crate::renderer::TextureId;

#[derive(Default, Clone)]
pub struct RendererConfig {
  pub device: DeviceConfig<'static>,
  pub graphics: GraphicsConfig,
}

impl RendererConfig {
  #[inline]
  pub fn set_device_config(mut self, device: DeviceConfig<'static>) -> Self {
    self.device = device;
    self
  }
}

#[derive(Default, Clone)]
pub struct DeviceConfig<'a> {
  pub layers: Vec<&'a std::ffi::CStr>,
  pub instance_extensions: Vec<&'a std::ffi::CStr>,
  pub device_extensions: Vec<&'a std::ffi::CStr>,
  pub device_features: vk::PhysicalDeviceFeatures,
}

impl<'a> DeviceConfig<'a> {
  #[inline]
  pub fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }
}

#[derive(Clone)]
pub struct GraphicsConfig {
  pub(crate) textures: Vec<(Vec<u8>, vk::Filter)>,
  max_texture_id: u32,
}

impl GraphicsConfig {
  #[inline]
  pub fn add_texture(&mut self, texture: Vec<u8>, interpolation: vk::Filter) -> TextureId {
    self.textures.push((texture, interpolation));
    let id = TextureId(self.max_texture_id);
    self.max_texture_id += 1;
    id
  }
}

impl Default for GraphicsConfig {
  fn default() -> Self {
    GraphicsConfig {
      textures: vec![(
        include_bytes!("../assets/default.png").to_vec(),
        vk::Filter::NEAREST,
      )],
      max_texture_id: 1,
    }
  }
}
