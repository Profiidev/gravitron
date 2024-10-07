use std::collections::HashMap;

use log::{trace, warn};

use crate::ecs::{systems::query::Query, systems::resources::ResMut};

use crate::ecs_resources::components::camera::Camera;
use crate::ecs_resources::components::renderer::MeshRenderer;
use crate::ecs_resources::components::transform::Transform;
use crate::vulkan::graphics::resources::model::InstanceData;
use crate::vulkan::Vulkan;
use crate::Id;

pub fn renderer(
  mut vulkan: ResMut<Vulkan>,
  to_render: Query<(&MeshRenderer, &Transform)>,
  camera: Query<&Camera>,
) {
  trace!("Executing MeshRenderer");

  let Some(camera) = camera.into_iter().next() else {
    warn!("No camera found. Can't render anything");
    return;
  };

  let vulkan = &mut *vulkan;
  vulkan.wait_for_draw_start();

  let mut models: HashMap<Id, Vec<InstanceData>> = HashMap::new();
  for (mesh_render, transform) in to_render {
    let instances = models.entry(mesh_render.model_id).or_default();
    let material = &mesh_render.material;
    instances.push(InstanceData::new(
      transform.matrix(),
      transform.inv_matrix(),
      material.color,
      material.metallic,
      material.roughness,
    ));
  }

  vulkan.update_camera(camera);
  vulkan.record_command_buffer(&models);
  vulkan.draw_frame(&models);
}
