use gravitron_plugin::{
  app::{AppBuilder, Build},
  stages::MainSystemStage,
  Plugin,
};
use systems::propagation::transform_propagate;

pub mod components;
mod systems;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_main_system_at_stage(transform_propagate, MainSystemStage::PostRender);
  }
}
