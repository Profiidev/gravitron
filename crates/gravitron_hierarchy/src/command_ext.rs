use gravitron_ecs::{
  commands::Commands,
  entity::IntoEntity,
  storage::{ComponentBox, Storage},
  tick::Tick,
  EntityId,
};

use crate::components::{Children, Parent};

pub trait HierarchyCommandExt {
  fn create_children(&mut self, entity: EntityId, child: impl IntoEntity) -> EntityId;
  fn set_parent(&mut self, entity: EntityId, parent: EntityId);
  fn remove_children(&mut self, entity: EntityId);
  fn remove_entity_with_children(&mut self, entity: EntityId);
}

impl HierarchyCommandExt for Commands {
  #[inline]
  fn create_children(&mut self, entity: EntityId, child: impl IntoEntity) -> EntityId {
    let id = self.create_entity(child);
    self.add_comp(id, Parent(entity));

    self.custom_fn_command(move |storage, tick| {
      add_self_to_parent(storage, id, entity, tick);
    });

    id
  }

  #[inline]
  fn set_parent(&mut self, entity: EntityId, new_parent: EntityId) {
    self.custom_fn_command(move |storage, tick| {
      remove_self_from_parent(storage, entity, tick);
      add_self_to_parent(storage, entity, new_parent, tick);
    });

    self.remove_comp::<Parent>(entity);
    self.add_comp(entity, Parent(new_parent));
  }

  #[inline]
  fn remove_children(&mut self, entity: EntityId) {
    self.custom_fn_command(move |storage, tick| {
      remove_children_recursive(storage, entity, tick);
    });
  }

  #[inline]
  fn remove_entity_with_children(&mut self, entity: EntityId) {
    self.remove_children(entity);

    self.custom_fn_command(move |storage, tick| {
      remove_self_from_parent(storage, entity, tick);
    });

    self.remove_entity(entity);
  }
}

fn remove_children_recursive(storage: &mut Storage, entity: EntityId, tick: Tick) {
  if let Some(children) = storage.remove_comp::<Children>(entity, tick) {
    for id in children.children() {
      remove_children_recursive(storage, *id, tick);
      storage.remove_entity(*id);
    }
  }
}

fn remove_self_from_parent(storage: &mut Storage, entity: EntityId, tick: Tick) {
  if let Some(parent) = storage.remove_comp::<Parent>(entity, tick) {
    let old_parent = parent.0;

    if let Some(children) = storage.get_comp::<Children>(old_parent) {
      if children.0.len() == 1 {
        storage.remove_comp::<Children>(entity, tick);
      } else {
        children.0.retain(|id| *id != entity);
      }
    }
  }
}

fn add_self_to_parent(storage: &mut Storage, entity: EntityId, parent: EntityId, tick: Tick) {
  if let Some(children) = storage.get_comp::<Children>(parent) {
    children.0.push(entity);
  } else {
    storage.add_comp(parent, ComponentBox::new(Children(vec![entity]), tick));
  }
}
