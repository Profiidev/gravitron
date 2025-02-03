use crate::pipeline::manager::GraphicsPipelineId;

#[derive(Default)]
pub struct Material {
  pub color: glam::Vec4,
  pub texture_id: u32,
  pub metallic: f32,
  pub roughness: f32,
  pub shader: GraphicsPipelineId,
}

impl Material {
  pub fn new() -> Self {
    Self::default()
  }
}
