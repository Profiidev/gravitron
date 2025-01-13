use std::{any::TypeId, fmt::Display, hash::Hash};

use entity::IntoEntity;
use scheduler::{Scheduler, SchedulerBuilder};
use systems::{IntoSystem, System};
use world::{UnsafeWorldCell, World};

pub mod commands;
pub mod components;
pub mod entity;
pub(crate) mod scheduler;
pub(crate) mod storage;
pub mod systems;
pub(crate) mod world;

pub use gravitron_ecs_macros::Component;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Default)]
pub struct Id(pub(crate) u64);

impl Display for Id {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub type ComponentId = TypeId;
pub type EntityId = Id;
type ArchetypeId = Id;
type SystemId = Id;

pub struct ECS {
  scheduler: Scheduler,
  world: World,
}

pub struct ECSBuilder<K: Ord + Hash + Clone = usize> {
  scheduler: SchedulerBuilder<K>,
  world: World,
  sync_system_exec: bool,
}

impl ECS {
  #[inline]
  pub fn builder() -> ECSBuilder {
    ECSBuilder::new()
  }

  #[inline]
  pub fn run(&mut self) {
    self.scheduler.run(&mut self.world);
  }

  #[inline]
  pub fn set_resource<R: 'static>(&mut self, res: R) {
    self.world.set_resource(res);
  }

  #[inline]
  pub fn get_resource<R: 'static>(&mut self) -> Option<&R> {
    self.world.get_resource()
  }

  #[inline]
  pub fn get_resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
    self.world.get_resource_mut()
  }

  #[inline]
  pub fn get_world_cell(&mut self) -> UnsafeWorldCell<'static> {
    UnsafeWorldCell::new(&mut self.world)
  }
}

impl<K: Ord + Hash + Clone> ECSBuilder<K> {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn sync_system_exec(&mut self, value: bool) {
    self.sync_system_exec = value;
  }

  #[inline]
  pub fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System = S>) {
    self.scheduler.add_system(system);
  }

  #[inline]
  pub fn add_system_at_stage<I, S: System + 'static>(
    &mut self,
    system: impl IntoSystem<I, System = S>,
    relative_stage: K,
  ) {
    self.scheduler.add_system_at_stage(system, relative_stage);
  }

  #[inline]
  pub fn add_resource<R: 'static>(&mut self, res: R) {
    self.world.add_resource(res);
  }

  #[inline]
  pub fn create_entity(&mut self, entity: impl IntoEntity) -> EntityId {
    self.world.create_entity(entity)
  }

  #[inline]
  pub fn build(self) -> ECS {
    ECS {
      scheduler: self.scheduler.build(self.sync_system_exec),
      world: self.world,
    }
  }
}

impl<K: Ord + Clone + Hash> Default for ECSBuilder<K> {
  fn default() -> Self {
    Self {
      scheduler: Default::default(),
      world: Default::default(),
      sync_system_exec: Default::default(),
    }
  }
}

#[cfg(test)]
mod test {
  use gravitron_ecs_macros::Component;

  use crate as gravitron_ecs;
  use crate::systems::resources::{Res, ResMut};
  use crate::{commands::Commands, systems::query::Query, ECS};

  #[derive(Component)]
  struct A {
    x: usize,
  }

  #[derive(Component)]
  struct B {
    y: usize,
  }

  #[test]
  fn full() {
    fn system(q: Query<(&mut A, &B)>, cmds: &mut Commands) {
      for (_, a, b) in q {
        a.x += b.y;
      }
      cmds.create_entity(B { y: 1 });
    }

    let mut ecs = ECS::builder();
    ecs.add_system(system);

    for i in 0..10 {
      ecs.create_entity(A { x: i });
    }

    let mut ecs = ecs.build();

    for _ in 0..10 {
      ecs.run();
    }
  }

  #[test]
  #[should_panic]
  fn wrong_query() {
    fn system(_: Query<(&mut A, &mut A, &B)>) {}
    ECS::builder().add_system(system);
  }

  #[test]
  #[should_panic]
  fn wrong_res() {
    fn system(_: Res<i32>, _: ResMut<i32>) {}
    ECS::builder().add_system(system);
  }

  #[test]
  #[should_panic]
  fn wrong_cmds() {
    fn system(_: &mut Commands, _: &mut Commands) {}
    ECS::builder().add_system(system);
  }
}
