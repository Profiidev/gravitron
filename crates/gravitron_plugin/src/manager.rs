use log::info;

use crate::{
  app::{App, AppBuilder},
  Plugin,
};

#[derive(Default)]
pub struct PluginManager {
  plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_plugin(&mut self, plugin: impl Plugin) {
    self.plugins.push(Box::new(plugin));
  }

  pub fn build(&self) -> App {
    let mut builder = AppBuilder::new();

    for plugin in &self.plugins {
      info!("Running build for Plugin {}", plugin.name());
      plugin.build(&mut builder);
    }

    let mut builder = builder.finalize();

    for plugin in &self.plugins {
      info!("Running finalize for Plugin {}", plugin.name());
      plugin.finalize(&mut builder);
    }

    builder.build()
  }

  pub fn cleanup(&self, app: &mut App) {
    for plugin in &self.plugins {
      info!("Running cleanup for Plugin {}", plugin.name());
      plugin.cleanup(app);
    }
  }
}
