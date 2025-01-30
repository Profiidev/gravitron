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
    debug!("Adding Plugin {}", plugin.id().0);

    if self.plugins.iter().any(|p| p.id() == plugin.id()) {
      panic!("Error: can not add the plugin {} twice", plugin.id().0);
    }

    for dep in plugin.dependencies() {
      if !self.plugins.iter().map(|p| p.id()).any(|p| p == dep) {
        panic!("Error: the plugin {} needs to be added before the plugin {}!", dep.0, plugin.id().0);
      }
    }

    self.plugins.push(Box::new(plugin));
  }

  pub fn build(&self) -> App<Running> {
    let mut builder = AppBuilder::new();

    for plugin in &self.plugins {
      debug!("Running build for Plugin {}", plugin.id().0);
      plugin.build(&mut builder);
    }

    let mut builder = builder.finalize();

    for plugin in &self.plugins {
      debug!("Running finalize for Plugin {}", plugin.id().0);
      plugin.finalize(&mut builder);
    }

    builder.build()
  }

  pub fn cleanup(&self, app: &mut App<Cleanup>) {
    for plugin in &self.plugins {
      debug!("Running cleanup for Plugin {}", plugin.id().0);
      plugin.cleanup(app);
    }
  }
}
