use std::time::{Duration, Instant};

use log::{debug, trace};

use crate::{
  app::{App, AppBuilder, Cleanup, Running},
  ecs::resources::{engine_commands::EngineCommands, engine_info::EngineInfo},
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

  pub fn run(&self, app: &mut App<Running>) {
    let config = app.get_config();

    let mut last_frame = Instant::now();
    let frame_time = Duration::from_secs(1) / config.engine.fps;

    loop {
      let elapsed = last_frame.elapsed();

      if elapsed > frame_time {
        app.set_resource(EngineInfo {
          delta_time: elapsed.as_secs_f32(),
        });

        last_frame = Instant::now();

        app.run_main();

        let cmds = app
          .get_resource::<EngineCommands>()
          .expect("Failed to get Engine Commands");
        if cmds.is_shutdown() {
          debug!("Exiting game loop");
          break;
        }

        trace!("Frame took {:?}", last_frame.elapsed());
      }
    }
  }

  pub fn cleanup(&self, app: &mut App<Cleanup>) {
    for plugin in &self.plugins {
      debug!("Running cleanup for Plugin {}", plugin.name());
      plugin.cleanup(app);
    }
  }
}
