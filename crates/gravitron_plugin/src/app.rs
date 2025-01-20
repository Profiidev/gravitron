use std::marker::PhantomData;

use gravitron_ecs::{
  scheduler::{Scheduler, SchedulerBuilder},
  systems::{IntoSystem, System},
  world::World,
};

use crate::{
  config::AppConfig,
  stages::{CleanupSystemStage, InitSystemStage, MainSystemStage},
};

pub struct AppBuilder<S: Stage> {
  world: World,
  init_scheduler: SchedulerBuilder<InitSystemStage>,
  main_scheduler: SchedulerBuilder<MainSystemStage>,
  cleanup_scheduler: SchedulerBuilder<CleanupSystemStage>,
  config: AppConfig,
  marker: PhantomData<S>,
}

pub struct App {
  world: World,
  init_scheduler: Scheduler,
  main_scheduler: Scheduler,
  cleanup_scheduler: Scheduler,
}

impl App {
  pub fn get_resource<R: 'static>(&self) -> Option<&R> {
    self.world.get_resource()
  }

  pub fn get_resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
    self.world.get_resource_mut()
  }
}

impl<S: Stage> AppBuilder<S> {
  pub fn add_init_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.init_scheduler.add_system(system);
  }

  pub fn add_init_system_at_stage<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
    stage: InitSystemStage,
  ) {
    self.init_scheduler.add_system_at_stage(system, stage);
  }

  pub fn add_main_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.main_scheduler.add_system(system);
  }

  pub fn add_main_system_at_stage<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
    stage: MainSystemStage,
  ) {
    self.main_scheduler.add_system_at_stage(system, stage);
  }

  pub fn add_cleanup_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.cleanup_scheduler.add_system(system);
  }

  pub fn add_cleanup_system_at_stage<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
    stage: CleanupSystemStage,
  ) {
    self.cleanup_scheduler.add_system_at_stage(system, stage);
  }

  pub fn add_resource<R: 'static>(&mut self, res: R) {
    self.world.add_resource(res);
  }

  pub fn config(&self) -> &AppConfig {
    &self.config
  }

  pub(crate) fn build(mut self) -> App {
    self.world.add_resource(self.config);

    App {
      world: self.world,
      init_scheduler: self.init_scheduler.build(false),
      main_scheduler: self.main_scheduler.build(false),
      cleanup_scheduler: self.cleanup_scheduler.build(false),
    }
  }
}

impl AppBuilder<Build> {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub fn config_mut(&mut self) -> &mut AppConfig {
    &mut self.config
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
  pub fn get_resource<R: 'static>(&self) -> Option<&R> {
    self.world.get_resource()
  }

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

    Self {
      world: Default::default(),
      init_scheduler: Default::default(),
      main_scheduler: Default::default(),
      cleanup_scheduler: Default::default(),
      config: Default::default(),
      marker: PhantomData,
    }
  }
}

pub trait Stage {}

pub struct Build {}
impl Stage for Build {}

pub struct Finalize {}
impl Stage for Finalize {}
