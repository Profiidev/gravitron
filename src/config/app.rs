pub struct AppConfig {
  pub fps: u32,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      fps: 60,
    }
  }
}
