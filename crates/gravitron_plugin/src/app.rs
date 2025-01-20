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

pub struct App<S: Status> {
  world: World,
  init_scheduler: Scheduler,
  main_scheduler: Scheduler,
  cleanup_scheduler: Scheduler,
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
  pub fn run_main(&mut self) {
    self.main_scheduler.run(&mut self.world);
  }

  #[inline]
  pub fn run_cleanup(&mut self) {
    self.cleanup_scheduler.run(&mut self.world);
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
  pub fn config(&self) -> &AppConfig {
    &self.config
  }

  pub(crate) fn build(self) -> App<Running> {
    App {
      world: self.world,
      init_scheduler: self.init_scheduler.build(false),
      main_scheduler: self.main_scheduler.build(false),
      cleanup_scheduler: self.cleanup_scheduler.build(false),
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

pub trait Status {}

pub struct Running {}
impl Status for Running {}

pub struct Cleanup {}
impl Status for Cleanup {}
