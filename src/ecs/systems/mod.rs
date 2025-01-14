use log::debug;
use renderer::{execute_renderer, init_renderer, renderer_recording};
use stages::SystemStage;

pub use gravitron_ecs::systems::*;

use super::ECSBuilder;

mod renderer;
pub mod stages;

pub fn add_main_systems(ecs: &mut ECSBuilder<SystemStage>) {
  debug!("Adding Engine Systems");

  ecs
    .main_scheduler
    .add_system_at_stage(init_renderer, SystemStage::RenderInit);
  ecs
    .main_scheduler
    .add_system_at_stage(renderer_recording, SystemStage::RenderRecording);
  ecs
    .main_scheduler
    .add_system_at_stage(execute_renderer, SystemStage::RenderExecute);
}
