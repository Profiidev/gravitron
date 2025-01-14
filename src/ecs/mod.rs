pub mod components;
pub mod resources;
pub mod systems;

use std::hash::Hash;

pub use gravitron_ecs::{commands, Component, ComponentId, EntityId, Id};
use gravitron_ecs::{
  scheduler::{Scheduler, SchedulerBuilder},
  world::World,
};

pub struct ECS {
  pub(crate) world: World,
  pub(crate) main_scheduler: Scheduler,
}

pub struct ECSBuilder<T: Ord + Hash + Clone> {
  pub(crate) world: World,
  pub(crate) main_scheduler: SchedulerBuilder<T>,
}

impl<T: Ord + Hash + Clone> Default for ECSBuilder<T> {
  fn default() -> Self {
    ECSBuilder {
      world: Default::default(),
      main_scheduler: Default::default(),
    }
  }
}
