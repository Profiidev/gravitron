use std::collections::HashMap;

#[allow(unused_imports)]
use log::{trace, warn};

use crate::ecs::{systems::query::Query, systems::resources::ResMut};

use crate::ecs::components::camera::Camera;
use crate::ecs::components::renderer::MeshRenderer;
use crate::ecs::components::transform::Transform;
use crate::vulkan::graphics::resources::model::{InstanceData, ModelId};
use crate::vulkan::Vulkan;

pub fn init_renderer(vulkan: ResMut<Vulkan>) {
  #[cfg(feature = "debug")]
  trace!("Initializing Renderer");
  vulkan.wait_for_draw_start();
}

pub fn renderer_recording(
  mut vulkan: ResMut<Vulkan>,
  to_render: Query<(&MeshRenderer, &Transform)>,
  camera: Query<&Camera>,
) {
  #[cfg(feature = "debug")]
  trace!("Recording Render Instructions");

  if let Some(camera) = camera.into_iter().next() {
    vulkan.update_camera(camera);
  } else {
    warn!("No camera found. Can't render anything");
    return;
  };

  let mut models: HashMap<ModelId, HashMap<String, Vec<InstanceData>>> = HashMap::new();
  for (mesh_render, transform) in to_render {
    let shader = models.entry(mesh_render.model_id).or_default();
    let instances = shader
      .entry(mesh_render.material.shader.clone())
      .or_default();
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

  vulkan.update_draw_info(models);
}

pub fn execute_renderer(mut vulkan: ResMut<Vulkan>) {
  #[cfg(feature = "debug")]
  trace!("Drawing Frame");
  vulkan.draw_frame();
}
