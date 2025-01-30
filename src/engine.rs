use gravitron_components::ComponentPlugin;
use gravitron_plugin::{
  app::{App, Running},
  manager::PluginManager,
  Plugin,
};
use gravitron_renderer::RendererPlugin;
use gravitron_window::WindowPlugin;
use log::info;

pub struct GravitronBuilder {
  plugin_manager: PluginManager,
}

pub struct Gravitron {
  plugin_manager: PluginManager,
  app: App<Running>,
}

impl Gravitron {
  #[inline]
  pub fn builder() -> GravitronBuilder {
    GravitronBuilder::default()
  }

  pub fn run(mut self) -> ! {
    info!("Running Gravitron");
    self.app.run_init();
    self.app.run_main();

    info!("Cleaning up Gravitron");
    let mut app = self.app.run_cleanup();

    self.plugin_manager.cleanup(&mut app);

    std::process::exit(0);
  }
}

impl GravitronBuilder {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn add_plugin(&mut self, plugin: impl Plugin) {
    self.plugin_manager.add_plugin(plugin);
  }

  pub fn build(self) -> Gravitron {
    info!("Building Gravitron");
    let app = self.plugin_manager.build();

    Gravitron {
      plugin_manager: self.plugin_manager,
      app,
    }
  }
}

impl Default for GravitronBuilder {
  fn default() -> Self {
    env_logger::init();

    info!("Creating PluginManager");
    let mut plugin_manager = PluginManager::new();

    info!("Adding default plugins");
    plugin_manager.add_plugin(ComponentPlugin);
    plugin_manager.add_plugin(WindowPlugin);
    plugin_manager.add_plugin(RendererPlugin);

    Self { plugin_manager }
  }
}
