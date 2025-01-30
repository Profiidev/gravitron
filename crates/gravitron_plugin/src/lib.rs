use std::any::TypeId;

use app::{App, AppBuilder, Build, Cleanup, Finalize};

pub mod app;
pub mod config;
pub mod ecs;
pub mod manager;
pub mod stages;

#[derive(Clone, Copy)]
pub struct PluginID(pub(crate) &'static str, TypeId);

impl PartialEq for PluginID {
  fn eq(&self, other: &Self) -> bool {
    self.1 == other.1
  }
}

pub trait Plugin: 'static {
  fn build(&self, _builder: &mut AppBuilder<Build>) {}
  fn finalize(&self, _builder: &mut AppBuilder<Finalize>) {}
  fn cleanup(&self, _app: &mut App<Cleanup>) {}

  fn id(&self) -> PluginID {
    PluginID(std::any::type_name::<Self>(), TypeId::of::<Self>())
  }
  fn dependencies(&self) -> Vec<PluginID> {
    vec![]
  }
}
