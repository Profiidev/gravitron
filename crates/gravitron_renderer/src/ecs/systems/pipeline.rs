use gravitron_ecs::systems::resources::ResMut;

use crate::pipeline::PipelineManager;

pub fn pipeline_changed_reset(mut pipeline_manager: ResMut<PipelineManager>) {
  pipeline_manager.graphics_changed_reset();
}
