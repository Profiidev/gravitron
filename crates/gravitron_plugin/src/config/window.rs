#[derive(Clone)]
pub struct WindowConfig {
  pub title: String,
  pub width: u32,
  pub height: u32,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      title: "Gravitron".into(),
      width: 800,
      height: 600,
    }
  }
}
