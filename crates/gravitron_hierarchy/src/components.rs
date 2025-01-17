use gravitron_ecs::{Component, EntityId};

#[derive(Component)]
pub struct Parent(pub(crate) EntityId);

impl Parent {
  pub fn parent(&self) -> EntityId {
    self.0
  }
}

#[derive(Component)]
pub struct Children(pub(crate) Vec<EntityId>);

impl Children {
  pub fn children(&self) -> &[EntityId] {
    &self.0
  }
}
