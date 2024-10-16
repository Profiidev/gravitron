pub struct Material {
  pub color: glam::Vec3,
  pub texture_id: u32,
  pub metallic: f32,
  pub roughness: f32,
  pub shader: String,
}

impl Material {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for Material {
  fn default() -> Self {
    Self {
      color: Default::default(),
      texture_id: 0,
      metallic: Default::default(),
      roughness: Default::default(),
      shader: "default".into(),
    }
  }
}
