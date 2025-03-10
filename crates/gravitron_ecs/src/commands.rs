use std::marker::PhantomData;

#[cfg(feature = "debug")]
use log::trace;

use crate::{
  components::Component,
  entity::IntoEntity,
  storage::{ComponentBox, Storage},
  systems::{metadata::SystemMeta, SystemParam},
  tick::Tick,
  world::UnsafeWorldCell,
  EntityId, SystemId,
};

pub struct Commands {
  commands: Vec<Box<dyn Command>>,
  world: UnsafeWorldCell<'static>,
}

impl Commands {
  pub(crate) fn create(world: UnsafeWorldCell<'static>) -> Self {
    Commands {
      world,
      commands: Vec::new(),
    }
  }

  pub(crate) fn execute(&mut self, storage: &mut Storage, tick: Tick) {
    for mut cmd in std::mem::take(&mut self.commands) {
      cmd.execute(storage, tick)
    }
  }

  pub fn create_entity(&mut self, entity: impl IntoEntity) -> EntityId {
    #[cfg(feature = "debug")]
    trace!("Registering Create Entity Command");

    let id = unsafe { self.world.world_mut() }.reserve_entity_id();

    self.commands.push(Box::new(CreateEntityCommand {
      comps: Some(entity.into_entity()),
      id,
    }));

    id
  }

  pub fn remove_entity(&mut self, entity: EntityId) {
    #[cfg(feature = "debug")]
    trace!("Registering Remove Entity Command for Entity {}", entity);

    self
      .commands
      .push(Box::new(RemoveEntityCommand { id: entity }));
  }

  pub fn add_comp(&mut self, entity: EntityId, comp: impl Component) {
    #[cfg(feature = "debug")]
    trace!(
      "Registering Add Component Command for Entity {} with Component {:?}",
      entity,
      comp.id()
    );

    self.commands.push(Box::new(AddComponentCommand {
      id: entity,
      comp: Some(Box::new(comp)),
    }));
  }

  pub fn remove_comp<C: Component>(&mut self, entity: EntityId) {
    #[cfg(feature = "debug")]
    trace!(
      "Registering Remove Component Command for Entity {} with Component {:?}",
      entity,
      C::sid()
    );

    self.commands.push(Box::new(RemoveComponentCommand {
      id: entity,
      phantom: PhantomData::<C>,
    }));
  }

  pub fn custom_fn_command<F>(&mut self, func: F)
  where
    F: Fn(&mut Storage, Tick) + 'static,
  {
    #[cfg(feature = "debug")]
    trace!("Registering Custom Fn Command",);

    self.commands.push(Box::new(CustomFnCommand { func }));
  }
}

impl SystemParam for &mut Commands {
  type Item<'new> = &'new mut Commands;

  #[inline]
  fn get_param(world: crate::world::UnsafeWorldCell<'_>, id: SystemId) -> Self::Item<'_> {
    unsafe { world.world_mut() }.get_commands_mut(id)
  }

  #[inline]
  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_cmds();
  }
}

trait Command {
  fn execute(&mut self, storage: &mut Storage, tick: Tick);
}

struct CreateEntityCommand {
  comps: Option<Vec<Box<dyn Component>>>,
  id: EntityId,
}

impl Command for CreateEntityCommand {
  fn execute(&mut self, storage: &mut Storage, tick: Tick) {
    #[cfg(feature = "debug")]
    trace!("Executing Create Entity Command");

    storage.create_entity_with_id(std::mem::take(&mut self.comps).unwrap(), self.id, tick);
  }
}

struct RemoveEntityCommand {
  id: EntityId,
}

impl Command for RemoveEntityCommand {
  fn execute(&mut self, storage: &mut Storage, _: Tick) {
    #[cfg(feature = "debug")]
    trace!("Executing Remove Entity Command for Entity {}", self.id);

    storage.remove_entity(self.id);
  }
}

struct AddComponentCommand {
  id: EntityId,
  comp: Option<Box<dyn Component>>,
}

impl Command for AddComponentCommand {
  fn execute(&mut self, storage: &mut Storage, tick: Tick) {
    #[cfg(feature = "debug")]
    trace!(
      "Executing Add Component Command for Entity {} with Component {:?}",
      self.id,
      self.comp.as_ref().unwrap().id()
    );

    storage.add_comp(
      self.id,
      ComponentBox {
        comp: std::mem::take(&mut self.comp).unwrap(),
        added: tick,
        changed: (Tick::INVALID, Tick::INVALID),
      },
    );
  }
}

struct RemoveComponentCommand<C: Component> {
  id: EntityId,
  phantom: PhantomData<C>,
}

impl<C: Component> Command for RemoveComponentCommand<C> {
  fn execute(&mut self, storage: &mut Storage, tick: Tick) {
    #[cfg(feature = "debug")]
    trace!(
      "Executing Remove Component Command for Entity {} with Component {:?}",
      self.id,
      C::sid()
    );

    storage.remove_comp::<C>(self.id, tick);
  }
}

struct CustomFnCommand<F>
where
  F: Fn(&mut Storage, Tick),
{
  func: F,
}

impl<F> Command for CustomFnCommand<F>
where
  F: Fn(&mut Storage, Tick),
{
  fn execute(&mut self, storage: &mut Storage, tick: Tick) {
    (self.func)(storage, tick)
  }
}

#[cfg(test)]
mod test {
  use super::Commands;
  use crate::{
    self as gravitron_ecs,
    world::{UnsafeWorldCell, World},
    Id,
  };
  use gravitron_ecs_macros::Component;

  #[derive(Component)]
  struct A {}

  #[test]
  fn create_entity() {
    let mut world = World::default();
    let mut commands = Commands::create(UnsafeWorldCell::new(&mut world));

    commands.create_entity(A {});
  }

  #[test]
  fn remove_entity() {
    let mut world = World::default();
    let mut commands = Commands::create(UnsafeWorldCell::new(&mut world));

    commands.remove_entity(Id(0));
  }

  #[test]
  fn add_comp() {
    let mut world = World::default();
    let mut commands = Commands::create(UnsafeWorldCell::new(&mut world));

    commands.add_comp(Id(0), A {});
  }

  #[test]
  fn remove_comp() {
    let mut world = World::default();
    let mut commands = Commands::create(UnsafeWorldCell::new(&mut world));

    commands.remove_comp::<A>(Id(0));
  }
}
