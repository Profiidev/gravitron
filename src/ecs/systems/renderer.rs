use std::collections::HashMap;

#[allow(unused_imports)]
use log::{trace, warn};

use crate::ecs::components::camera::Camera;
use crate::ecs::components::lighting::{
  DirectionalLight as DirectionalLightComp, PointLight as PointLightComp,
  SpotLight as SpotLightComp,
};
use crate::ecs::components::renderer::MeshRenderer;
use crate::ecs::components::transform::Transform;
use crate::ecs::{systems::query::Query, systems::resources::ResMut};

use crate::vulkan::graphics::resources::lighting::{
  DirectionalLight, LightInfo, PointLight, SpotLight,
};
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
  dl_query: Query<(&DirectionalLightComp, &Transform)>,
  pls_query: Query<(&PointLightComp, &Transform)>,
  sls_query: Query<(&SpotLightComp, &Transform)>,
  camera: Query<&Camera>,
) {
  #[cfg(feature = "debug")]
  trace!("Recording Render Instructions");

  if let Some((_, camera)) = camera.into_iter().next() {
    vulkan.update_camera(camera);
  } else {
    warn!("No camera found. Can't render anything");
    return;
  };

  let mut models: HashMap<ModelId, HashMap<String, Vec<InstanceData>>> = HashMap::new();
  for (_, mesh_render, transform) in to_render {
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

  let mut pls = Vec::new();
  for (_, pl, t) in pls_query {
    pls.push(PointLight {
      position: t.position().into(),
      color: pl.color,
      intensity: pl.intensity,
      range: pl.range,
    });
  }
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

  vulkan.update_draw_info(models, light_info, &pls, &sls);
}

pub fn execute_renderer(mut vulkan: ResMut<Vulkan>) {
  #[cfg(feature = "debug")]
  trace!("Drawing Frame");
  vulkan.draw_frame();
}
