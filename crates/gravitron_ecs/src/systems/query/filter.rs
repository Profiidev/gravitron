use crate::{storage::Row, ComponentId};

pub trait QueryFilter {
  fn filter_archetype(r#type: &[ComponentId]) -> bool;
  fn filter_entity(entity: &Row) -> bool;
}

pub trait QueryFilterParam {
  fn filter_archetype(r#type: &[ComponentId]) -> bool;
  fn filter_entity(entity: &Row) -> bool;
}

impl QueryFilter for () {
  fn filter_archetype(_: &[ComponentId]) -> bool {
    true
  }

  fn filter_entity(_: &Row) -> bool {
    true
  }
}
