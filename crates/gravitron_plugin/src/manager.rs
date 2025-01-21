use log::debug;

use crate::{
  app::{App, AppBuilder, Cleanup, Running},
  Plugin,
};

#[derive(Default)]
pub struct PluginManager {
  plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn add_plugin(&mut self, plugin: impl Plugin) {
    debug!("Adding Plugin {}", plugin.name());
    self.plugins.push(Box::new(plugin));
  }

  pub fn build(&self) -> App<Running> {
    let mut builder = AppBuilder::new();

    for plugin in &self.plugins {
      debug!("Running build for Plugin {}", plugin.name());
      plugin.build(&mut builder);
    }

    let mut builder = builder.finalize();

    for plugin in &self.plugins {
      debug!("Running finalize for Plugin {}", plugin.name());
      plugin.finalize(&mut builder);
    }

    builder.build()
  }

  pub fn cleanup(&self, app: &mut App<Cleanup>) {
    for plugin in &self.plugins {
      debug!("Running cleanup for Plugin {}", plugin.name());
      plugin.cleanup(app);
    }
  }
}
