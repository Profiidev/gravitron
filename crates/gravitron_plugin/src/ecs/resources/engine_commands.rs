#[derive(Default)]
pub struct EngineCommands {
  shutdown: bool,
}

impl EngineCommands {
  pub fn shutdown(&mut self) {
    self.shutdown = true;
  }

  pub fn is_shutdown(&self) -> bool {
    self.shutdown
  }
}
