use std::{any::TypeId, fmt::Display, hash::Hash};

pub mod commands;
pub mod components;
pub mod entity;
pub mod scheduler;
pub(crate) mod storage;
pub mod systems;
pub mod tick;
pub mod world;

#[cfg(test)]
mod test;

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
