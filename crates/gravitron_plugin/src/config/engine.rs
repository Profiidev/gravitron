pub struct EngineConfig {
  pub version: u32,
  pub fps: u32,
  pub parallel_systems: bool,
}

impl Default for EngineConfig {
  fn default() -> Self {
    Self {
      version: 1,
      fps: 60,
      parallel_systems: true,
    }
  }
}
