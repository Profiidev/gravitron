#[derive(Clone)]
pub struct WindowConfig {
  pub title: String,
  pub width: u32,
  pub height: u32,
  pub version: u32,
  pub fps: u32,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      title: "Gravitron".into(),
      width: 800,
      height: 600,
      version: 1,
      fps: 60,
    }
  }
}
