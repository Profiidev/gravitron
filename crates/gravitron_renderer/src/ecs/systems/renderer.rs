use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use gravitron_ecs::systems::resources::Res;
#[cfg(feature = "debug")]
use log::trace;

use crate::ecs::components::renderer::MeshRenderer;
use crate::memory::MemoryManager;
use crate::model::model::{InstanceData, ModelId};
use crate::model::ModelManager;
use crate::pipeline::manager::GraphicsPipelineId;
use crate::pipeline::{DescriptorManager, PipelineManager};
use crate::renderer::Renderer;
use gravitron_components::components::transform::GlobalTransform;
use gravitron_ecs::{systems::query::Query, systems::resources::ResMut};

pub fn init_renderer(renderer: Res<Renderer>) {
  #[cfg(feature = "debug")]
  trace!("Initializing Renderer");
  renderer.wait_for_draw_start();
}

pub fn draw_data_update(
  mut renderer: ResMut<Renderer>,
  mut memory_manager: ResMut<MemoryManager>,
  mut model_manager: ResMut<ModelManager>,
  to_render: Query<(&MeshRenderer, &GlobalTransform)>,
) {
  #[cfg(feature = "debug")]
  trace!("Updating Renderer Buffers");

  let mut models: HashMap<ModelId, HashMap<GraphicsPipelineId, Vec<InstanceData>>> = HashMap::new();
  for (_, mesh_render, transform) in to_render {
    let shader = models.entry(mesh_render.model_id).or_default();
    let instances = shader.entry(mesh_render.material.shader).or_default();
    let material = &mesh_render.material;
    instances.push(InstanceData::new(
      transform.matrix(),
      transform.inv_matrix(),
      material.color,
      material.metallic,
      material.roughness,
      material.texture_id,
    ));
  }

  renderer.update_draw_buffer(
    memory_manager.deref_mut(),
    models,
    model_manager.deref_mut(),
  );
}

pub fn renderer_recording(
  mut renderer: ResMut<Renderer>,
  mut memory_manager: ResMut<MemoryManager>,
  model_manager: Res<ModelManager>,
  pipeline_manager: ResMut<PipelineManager>,
  descriptor_manager: ResMut<DescriptorManager>,
) {
  #[cfg(feature = "debug")]
  trace!("Recording Command Buffers");
  renderer
    .record_command_buffer(
      pipeline_manager.deref(),
      descriptor_manager.deref(),
      memory_manager.deref_mut(),
      model_manager.deref(),
    )
    .expect("Failed to record CommandBuffer");
}

pub fn execute_renderer(mut renderer: ResMut<Renderer>) {
  #[cfg(feature = "debug")]
  trace!("Drawing Frame");
  renderer.draw_frame();
}
