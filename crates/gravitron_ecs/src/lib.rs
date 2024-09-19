use entity::IntoEntity;
use scheduler::{Scheduler, SchedulerBuilder};
use systems::{IntoSystem, System};
use world::World;

pub mod components;
pub mod commands;
pub(crate) mod entity;
pub mod query;
pub mod systems;
pub(crate) mod world;
pub(crate) mod scheduler;
pub(crate) mod storage;

pub type Id = u64;
pub type ComponentId = Id;
pub type EntityId = Id;
type ArchetypeId = Id;
type SystemId = Id;

#[derive(Default)]
pub struct ECS {
  scheduler: Scheduler,
  world: World,
}

#[derive(Default)]
pub struct ECSBuilder {
  scheduler: SchedulerBuilder,
  world: World
}

impl ECS {
  pub fn builder() -> ECSBuilder {
    ECSBuilder::new()
  }

  pub fn run(&mut self) {
    self.scheduler.run(&mut self.world);
  }
}

impl ECSBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_system<I, S: System + 'static>(mut self, system: impl IntoSystem<I, System = S>) -> Self {
    self.scheduler.add_system(system);
    self
  }

  pub fn add_resource<R: 'static>(mut self, res: R) -> Self {
    self.world.add_resource(res);
    self
  }

  pub fn create_entity(&mut self, entity: impl IntoEntity) -> EntityId {
    self.world.create_entity(entity)
  }

  pub fn build(self) -> ECS {
    ECS {
      scheduler: self.scheduler.build(),
      world: self.world
    }
  }
}

#[cfg(test)]
mod test {
  use gravitron_ecs_macros::Component;

use crate::systems::{Res, ResMut};
use crate::{commands::Commands, query::Query, ECS};
  use crate as gravitron_ecs;

  #[derive(Component)]
  struct A {
    x: usize
  }

  #[derive(Component)]
  struct B {
    y: usize
  }

  #[test]
  fn full() {
    fn system(q: Query<(&mut A, &B)>, cmds: &mut Commands) {
      for (a, b) in q {
        a.x += b.y;
      }
      cmds.create_entity(B { y: 1 })
    }

    let mut ecs = ECS::builder().add_system(system);

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
    fn system(_: Query<(&mut A, &mut A, &B)>) {

    }
    ECS::builder().add_system(system);
  }

  #[test]
  #[should_panic]
  fn wrong_res() {
    fn system(_: Res<i32>, _: ResMut<i32>) {

    }
    ECS::builder().add_system(system);
  }

  #[test]
  #[should_panic]
  fn wrong_cmds() {
    fn system(_: &mut Commands, _: &mut Commands) {

    }
    ECS::builder().add_system(system);
  }
}

