use crate::{components::Component, entity::IntoEntity, storage::{ComponentId, EntityId, Storage}, systems::{metadata::SystemMeta, SystemId, SystemParam}};

#[derive(Default)]
pub struct Commands {
  commands: Vec<Box<dyn Command>>
}

impl Commands {
  pub fn new() -> Self {
    Commands::default()
  }

  pub fn execute(&mut self, storage: &mut Storage) {
    for cmd in &mut self.commands {
      cmd.execute(storage)
    }
    self.commands = Vec::new();
  }

  pub fn create_entity(&mut self, entity: impl IntoEntity) {
    self.commands.push(Box::new(CreateEntityCommand {
      comps: Some(entity.into_entity())
    }));
  }

  pub fn remove_entity(&mut self, entity: EntityId) {
    self.commands.push(Box::new(RemoveEntityCommand {
      id: entity
    }));
  }

  pub fn add_comp(&mut self, entity: EntityId, comp: impl Component) {
    self.commands.push(Box::new(AddComponentCommand {
      id: entity,
      comp: Some(Box::new(comp))
    }));
  }

  pub fn remove_comp(&mut self, entity: EntityId, comp: ComponentId) {
    self.commands.push(Box::new(RemoveComponentCommand {
      id: entity,
      comp
    }));
  }
}

impl SystemParam for &mut Commands {
  type Item<'new> = &'new mut Commands;

  fn get_param(world: crate::world::UnsafeWorldCell<'_>, id: SystemId) -> Self::Item<'_> {
    unsafe { world.world_mut() }.get_commands_mut(id)
  }

  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_cmds();
  }
}

trait Command {
  fn execute(&mut self, storage: &mut Storage);
}

struct CreateEntityCommand {
  comps: Option<Vec<Box<dyn Component>>>
}

impl Command for CreateEntityCommand {
  fn execute(&mut self, storage: &mut Storage) {
    storage.create_entity(std::mem::take(&mut self.comps).unwrap());
  }
}

struct RemoveEntityCommand {
  id: EntityId
}

impl Command for RemoveEntityCommand {
  fn execute(&mut self, storage: &mut Storage) {
    storage.remove_entity(self.id);
  }
}

struct AddComponentCommand {
  id: EntityId,
  comp: Option<Box<dyn Component>>
}

impl Command for AddComponentCommand {
  fn execute(&mut self, storage: &mut Storage) {
    storage.add_comp(self.id, std::mem::take(&mut self.comp).unwrap());
  }
}

struct RemoveComponentCommand {
  id: EntityId,
  comp: ComponentId
}

impl Command for RemoveComponentCommand {
  fn execute(&mut self, storage: &mut Storage) {
    storage.remove_comp(self.id, self.comp);
  }
}

