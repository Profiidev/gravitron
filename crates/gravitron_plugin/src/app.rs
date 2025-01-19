use std::marker::PhantomData;

use gravitron_ecs::{
  scheduler::{Scheduler, SchedulerBuilder},
  systems::{IntoSystem, System},
  world::World,
};

use crate::{config::window::WindowConfig, stages::SystemStage};

pub struct AppBuilder<S: Stage> {
  world: World,
  init_scheduler: SchedulerBuilder,
  main_scheduler: SchedulerBuilder<SystemStage>,
  cleanup_scheduler: SchedulerBuilder,
  config: WindowConfig,
  marker: PhantomData<S>,
}

pub struct App {
  init_scheduler: Scheduler,
  main_scheduler: Scheduler,
  cleanup_scheduler: Scheduler,
}

impl<S: Stage> AppBuilder<S> {
  pub fn add_init_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.init_scheduler.add_system(system);
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
    stage: SystemStage,
  ) {
    self.main_scheduler.add_system_at_stage(system, stage);
  }

  pub fn add_cleanup_system<I, Sy: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = Sy>,
  ) {
    self.cleanup_scheduler.add_system(system);
  }

  pub fn add_resource<R: 'static>(&mut self, res: R) {
    self.world.add_resource(res);
  }

  pub fn config(&self) -> &WindowConfig {
    &self.config
  }

  pub(crate) fn build(self) -> App {
    App {
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

  pub fn config_mut(&mut self) -> &mut WindowConfig {
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

impl Default for AppBuilder<Build> {
  fn default() -> Self {
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
