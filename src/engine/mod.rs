use std::{
  thread::{self, JoinHandle},
  time::{Duration, Instant},
};

use gravitron_ecs::{
  entity::IntoEntity,
  systems::{IntoSystem, System},
  ECSBuilder, EntityId, ECS,
};
use gravitron_utils::thread::Signal;
use log::info;
use window::Window;

use crate::{config::EngineConfig, systems::add_systems};

mod window;

pub struct Gravitron {
  ecs: ECS,
  fps: u32,
  window_handle: JoinHandle<()>,
  app_run: Signal,
}

pub struct GravitronBuilder {
  ecs: ECSBuilder,
  config: EngineConfig,
}

impl Gravitron {
  pub fn builder(config: EngineConfig) -> GravitronBuilder {
    GravitronBuilder::new(config)
  }

  pub fn run(mut self) -> ! {
    info!("Starting Engine");
    let mut last_frame = Instant::now();
    let time_per_frame = Duration::from_secs(1) / self.fps;

    self.app_run.signal();

    loop {
      if last_frame.elapsed() > time_per_frame {
        self.ecs.run();
        last_frame = Instant::now();
      }
    }
  }
}

impl GravitronBuilder {
  pub fn new(config: EngineConfig) -> Self {
    env_logger::init();

    GravitronBuilder {
      ecs: Default::default(),
      config,
    }
  }

  pub fn add_resource<R: 'static>(mut self, res: R) -> Self {
    self.ecs.add_resource(res);
    self
  }

  pub fn add_system<I, S: System + 'static>(
    mut self,
    system: impl IntoSystem<I, System = S>,
  ) -> Self {
    self.ecs.add_system(system);
    self
  }

  pub fn create_entity(&mut self, entity: impl IntoEntity) -> EntityId {
    self.ecs.create_entity(entity)
  }

  pub fn build(mut self) -> Gravitron {
    info!("Building Engine");
    let window_ready = Signal::new();
    let app_run = Signal::new();

    let thread_window_ready = Signal::clone_inner(&window_ready);
    let thread_app_run = app_run.clone();

    let fps = self.config.app.fps;

    let window_handle = thread::spawn(move || {
      info!("Creating Window");
      Window::init(self.config, thread_app_run, thread_window_ready).unwrap();
    });

    add_systems(&mut self.ecs);

    self.ecs.add_resource(window_ready.wait());

    Gravitron {
      ecs: self.ecs.build(),
      fps,
      window_handle,
      app_run,
    }
  }
}
