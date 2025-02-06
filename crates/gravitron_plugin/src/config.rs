pub struct AppConfig {
  pub version: u32,
  pub fps: u32,
  pub parallel_systems: bool,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      version: 1,
      fps: 60,
      parallel_systems: true,
    }
  }
}
