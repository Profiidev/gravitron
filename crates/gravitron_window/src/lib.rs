use gravitron_plugin::{app::{AppBuilder, Finalize}, Plugin};

pub mod ecs;

pub struct WindowPlugin {}

impl Plugin for WindowPlugin {
  fn finalize(&self, builder: &mut AppBuilder<Finalize>) {
  }
}
