use gravitron_ecs::ECSBuilder;
use log::debug;
use renderer::{execute_renderer, init_renderer, renderer_recording};
use stages::SystemStage;

pub use gravitron_ecs::systems::*;

mod renderer;
pub mod stages;

pub fn add_systems(ecs: &mut ECSBuilder<SystemStage>) {
  debug!("Adding Engine Systems");

  ecs.add_system_at_stage(init_renderer, SystemStage::RenderInit);
  ecs.add_system_at_stage(renderer_recording, SystemStage::RenderRecording);
  ecs.add_system_at_stage(execute_renderer, SystemStage::RenderExecute);
}
