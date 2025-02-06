use std::{
  any::{type_name, Any, TypeId},
  collections::HashMap,
  marker::PhantomData,
  time::{Duration, Instant},
};

use gravitron_ecs::{
  scheduler::{Scheduler, SchedulerBuilder},
  systems::{IntoSystem, System},
  world::World,
};
use log::debug;
#[cfg(feature = "debug")]
use log::trace;

use crate::{
  config::AppConfig,
  ecs::resources::{engine_commands::EngineCommands, engine_info::EngineInfo},
  stages::{CleanupSystemStage, InitSystemStage, MainSystemStage},
};

pub struct AppBuilder<S: Stage> {
  world: World,
  init_scheduler: SchedulerBuilder<InitSystemStage>,
  main_scheduler: SchedulerBuilder<MainSystemStage>,
  cleanup_scheduler: SchedulerBuilder<CleanupSystemStage>,
  config: HashMap<TypeId, Box<dyn Any>>,
  marker: PhantomData<S>,
}

pub struct App<S: Status> {
  world: World,
  init_scheduler: Scheduler,
  main_scheduler: Scheduler,
  cleanup_scheduler: Scheduler,
  config: HashMap<TypeId, Box<dyn Any>>,
  marker: PhantomData<S>,
}

impl<S: Status> App<S> {
  #[inline]
  pub fn get_resource<R: 'static>(&self) -> Option<&R> {
    self.world.get_resource()
  }

  #[inline]
  pub fn get_resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
    self.world.get_resource_mut()
  }
}

impl App<Running> {
  #[inline]
  pub fn set_resource<R: 'static>(&mut self, res: R) {
    self.world.set_resource(res);
  }

  #[inline]
  pub fn run_init(&mut self) {
    self.init_scheduler.run(&mut self.world);
  }

  #[inline]
  pub fn config<C: 'static>(&self) -> Option<&C> {
    self
      .config
      .get(&TypeId::of::<C>())
      .and_then(|c| c.downcast_ref())
  }

  pub fn run_main(&mut self) {
    let fps = self.config::<AppConfig>().unwrap().fps;

    let mut last_frame = Instant::now();
    let frame_time = Duration::from_secs(1) / fps;

    loop {
      let elapsed = last_frame.elapsed();

      if elapsed > frame_time {
        self.set_resource(EngineInfo {
          delta_time: elapsed.as_secs_f32(),
        });

        last_frame = Instant::now();

        self.main_scheduler.run(&mut self.world);

        let cmds = self
          .get_resource::<EngineCommands>()
          .expect("Failed to get Engine Commands");
        if cmds.is_shutdown() {
          debug!("Exiting game loop");
          break;
        }

        self.world.next_tick();

        #[cfg(feature = "debug")]
        trace!("Frame took {:?}", last_frame.elapsed());
      }
    }
  }

  #[inline]
  pub fn run_cleanup(mut self) -> App<Cleanup> {
    self.cleanup_scheduler.run(&mut self.world);

    App {
      world: self.world,
      init_scheduler: self.init_scheduler,
      main_scheduler: self.main_scheduler,
      cleanup_scheduler: self.cleanup_scheduler,
      config: self.config,
      marker: PhantomData,
    }
  }
}

impl<S: Stage> AppBuilder<S> {
  #[inline]
  pub fn add_init_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.init_scheduler.add_system(system);
  }

  #[inline]
  pub fn add_init_system_at_stage<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
    stage: InitSystemStage,
  ) {
    self.init_scheduler.add_system_at_stage(system, stage);
  }

  #[inline]
  pub fn add_main_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.main_scheduler.add_system(system);
  }

  #[inline]
  pub fn add_main_system_at_stage<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
    stage: MainSystemStage,
  ) {
    self.main_scheduler.add_system_at_stage(system, stage);
  }

  #[inline]
  pub fn add_cleanup_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.cleanup_scheduler.add_system(system);
  }

  #[inline]
  pub fn add_cleanup_system_at_stage<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
    stage: CleanupSystemStage,
  ) {
    self.cleanup_scheduler.add_system_at_stage(system, stage);
  }

  #[inline]
  pub fn add_resource<R: 'static>(&mut self, res: R) {
    self.world.add_resource(res);
  }

  #[inline]
  pub fn config<C: 'static>(&self) -> Option<&C> {
    self
      .config
      .get(&TypeId::of::<C>())
      .and_then(|c| c.downcast_ref())
  }

  pub(crate) fn build(mut self) -> App<Running> {
    self.world.add_resource(EngineCommands::default());
    let parallel = self.config::<AppConfig>().unwrap().parallel_systems;

    App {
      world: self.world,
      init_scheduler: self.init_scheduler.build(parallel),
      main_scheduler: self.main_scheduler.build(parallel),
      cleanup_scheduler: self.cleanup_scheduler.build(parallel),
      config: self.config,
      marker: PhantomData,
    }
  }
}

impl AppBuilder<Build> {
  #[inline]
  pub(crate) fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn config_mut<C: 'static>(&mut self) -> Option<&mut C> {
    self
      .config
      .get_mut(&TypeId::of::<C>())
      .and_then(|c| c.downcast_mut())
  }

  #[inline]
  pub fn add_config<C: 'static>(&mut self, config: C) {
    debug!("Adding Config {}", type_name::<C>());
    self.config.insert(TypeId::of::<C>(), Box::new(config));
  }

  pub(crate) fn finalize(self) -> AppBuilder<Finalize> {
    AppBuilder {
      world: self.world,
      init_scheduler: self.init_scheduler,
      main_scheduler: self.main_scheduler,
      cleanup_scheduler: self.cleanup_scheduler,
      config: self.config,
      marker: PhantomData,
    }
  }
}

impl AppBuilder<Finalize> {
  #[inline]
  pub fn get_resource<R: 'static>(&self) -> Option<&R> {
    self.world.get_resource()
  }

  #[inline]
  pub fn get_resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
    self.world.get_resource_mut()
  }
}

impl Default for AppBuilder<Build> {
  fn default() -> Self {
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
      orig_hook(panic_info);
      std::process::exit(1);
    }));

    let mut builder = Self {
      world: Default::default(),
      init_scheduler: Default::default(),
      main_scheduler: Default::default(),
      cleanup_scheduler: Default::default(),
      config: Default::default(),
      marker: PhantomData,
    };

    builder.add_config(AppConfig::default());

    builder
  }
}

pub trait Stage {}

pub struct Build {}
impl Stage for Build {}

pub struct Finalize {}
impl Stage for Finalize {}

pub trait Status {}

pub struct Running {}
impl Status for Running {}

pub struct Cleanup {}
impl Status for Cleanup {}
