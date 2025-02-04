use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use gravitron_ecs::systems::query::filter::Changed;
use gravitron_ecs::systems::resources::Res;
#[cfg(feature = "debug")]
use log::trace;

use crate::ecs::components::camera::Camera;
use crate::ecs::components::lighting::{
  DirectionalLight as DirectionalLightComp, PointLight as PointLightComp,
  SpotLight as SpotLightComp,
};
use crate::ecs::components::renderer::MeshRenderer;
use crate::memory::MemoryManager;
use crate::model::model::{InstanceData, ModelId};
use crate::model::ModelManager;
use crate::pipeline::manager::GraphicsPipelineId;
use crate::pipeline::DescriptorManager;
use crate::renderer::{Renderer, CAMERA_DESCRIPTOR};
use gravitron_components::components::transform::{GlobalTransform, Transform};
use gravitron_ecs::{systems::query::Query, systems::resources::ResMut};

use crate::renderer::resources::lighting::{DirectionalLight, LightInfo, PointLight, SpotLight};

pub fn init_renderer(renderer: Res<Renderer>) {
  #[cfg(feature = "debug")]
  trace!("Initializing Renderer");
  renderer.wait_for_draw_start();
}

pub fn update_descriptors(
  mut descriptor_manager: ResMut<DescriptorManager>,
  mut memory_manager: ResMut<MemoryManager>,
  camera: Query<(&mut Camera, &Transform), Changed<Transform>>,
  dl_query: Query<(&DirectionalLightComp, &GlobalTransform)>,
  pls_query: Query<(&PointLightComp, &GlobalTransform)>,
  sls_query: Query<(&SpotLightComp, &GlobalTransform)>,
) {
  if let Some((_, mut camera, transform)) = camera.into_iter().next() {
    camera.update_view_matrix(transform.deref());

    let camera_desc = descriptor_manager
      .descriptor(CAMERA_DESCRIPTOR)
      .expect("Failed to get Camera Descriptor");

    let camera_mem = camera_desc.uniform().expect("Camera not uniform");
    memory_manager
      .write_to_buffer(
        camera_mem,
        &[camera.projection_matrix(), camera.view_matrix()],
      )
      .expect("Failed to update Camera");
  }

  let mut pls = Vec::new();
  for (_, pl, t) in pls_query {
    pls.push(PointLight {
      position: t.position().into(),
      color: pl.color,
      intensity: pl.intensity,
      range: pl.range,
    });
  }

  let mut pls_desc = descriptor_manager
    .descriptor_mut(CAMERA_DESCRIPTOR)
    .expect("Failed to get PointLight Descriptor");

  let pls_mem = pls_desc.storage_mut().expect("PointLights not storage");
  if pls_mem.size() < size_of_val(&pls) {
    memory_manager
      .resize_buffer_mem(pls_mem, size_of_val(&pls))
      .expect("Failed to resize PointLights Mem");
  }
  memory_manager
    .write_to_buffer(pls_mem, &pls)
    .expect("Failed to write PointLights");

  let mut sls = Vec::new();
  for (_, sl, t) in sls_query {
    sls.push(SpotLight {
      position: t.position().into(),
      direction: (t.rotation() * glam::Vec3::X).into(),
      color: sl.color,
      intensity: sl.intensity,
      range: sl.range,
      angle: sl.angle,
    });
  }

  let mut sls_desc = descriptor_manager
    .descriptor_mut(CAMERA_DESCRIPTOR)
    .expect("Failed to get SpotLight Descriptor");

  let sls_mem = sls_desc.storage_mut().expect("SpotLights not storage");
  if sls_mem.size() < size_of_val(&sls) {
    memory_manager
      .resize_buffer_mem(sls_mem, size_of_val(&sls))
      .expect("Failed to resize SpotLights Mem");
  }
  memory_manager
    .write_to_buffer(sls_mem, &sls)
    .expect("Failed to write SpotLights");

  let dl = if let Some((_, dl, t)) = dl_query.into_iter().next() {
    DirectionalLight {
      direction: (t.rotation() * glam::Vec3::X).into(),
      color: dl.color,
      intensity: dl.intensity,
      ambient_color: dl.ambient_color,
      ambient_intensity: dl.ambient_intensity,
    }
  } else {
    DirectionalLight::default()
  };

  let light_info = LightInfo {
    num_point_lights: pls.len() as u32,
    num_spot_lights: sls.len() as u32,
    directional_light: dl,
  };

  let info_desc = descriptor_manager
    .descriptor(CAMERA_DESCRIPTOR)
    .expect("Failed to get LightInfo Descriptor");

  let info_mem = info_desc.uniform().expect("LightInfo not uniform");
  memory_manager
    .write_to_buffer(info_mem, &[light_info])
    .expect("Failed to update LightInfo");
}

pub fn renderer_recording(
  mut renderer: ResMut<Renderer>,
  mut memory_manager: ResMut<MemoryManager>,
  mut model_manager: ResMut<ModelManager>,
  to_render: Query<(&MeshRenderer, &GlobalTransform)>,
) {
  #[cfg(feature = "debug")]
  trace!("Recording Render Instructions");

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

pub fn execute_renderer(mut renderer: ResMut<Renderer>) {
  #[cfg(feature = "debug")]
  trace!("Drawing Frame");
  renderer.draw_frame();
}
