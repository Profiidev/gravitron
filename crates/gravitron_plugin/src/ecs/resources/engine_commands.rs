#[derive(Default)]
pub struct EngineCommands {
  shutdown: bool,
}

impl EngineCommands {
  #[inline]
  pub fn shutdown(&mut self) {
    self.shutdown = true;
  }

  #[inline]
  pub fn is_shutdown(&self) -> bool {
    self.shutdown
  }
}
