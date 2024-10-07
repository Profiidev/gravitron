pub struct Material {
  pub color: glam::Vec3,
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
      metallic: Default::default(),
      roughness: Default::default(),
      shader: "default".into(),
    }
  }
}
