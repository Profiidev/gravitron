pub mod resources;

use std::hash::Hash;

pub use gravitron_ecs::*;
pub use gravitron_hierarchy as hierarchy;

use scheduler::{Scheduler, SchedulerBuilder};
use world::World;

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
