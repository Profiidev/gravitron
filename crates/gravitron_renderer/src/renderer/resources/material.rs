use crate::{pipeline::manager::GraphicsPipelineId, renderer::TextureId};

pub struct Material {
  pub color: glam::Vec4,
  pub texture_id: TextureId,
  pub metallic: f32,
  pub roughness: f32,
  pub shader: GraphicsPipelineId,
}

impl Material {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for Material {
  fn default() -> Self {
    Self {
      color: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
      texture_id: Default::default(),
      metallic: 0.0,
      roughness: 0.0,
      shader: Default::default(),
    }
  }
}
