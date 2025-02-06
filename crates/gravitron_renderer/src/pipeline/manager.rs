use std::{collections::HashMap, ffi::CStr};

use anyhow::Error;
use ash::vk;

use crate::renderer::swapchain::SwapChain;

use super::{
  graphics::{stage::RenderingStage, GraphicsPipeline, GraphicsPipelineBuilder},
  DescriptorManager,
};

pub(crate) const MAIN_FN: &CStr = c"main";

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct GraphicsPipelineHandle(u64);

#[inline]
pub(crate) fn create_pipeline_cache(
  logical_device: &ash::Device,
  id: GraphicsPipelineHandle,
) -> Result<vk::PipelineCache, Error> {
  let initial_data = std::fs::read(format!("cache/{}.bin", id.0)).unwrap_or_default();

  let pipeline_cache_create_info =
    vk::PipelineCacheCreateInfo::default().initial_data(&initial_data);

  Ok(unsafe { logical_device.create_pipeline_cache(&pipeline_cache_create_info, None) }?)
}

#[inline]
pub(crate) unsafe fn cleanup_pipeline_cache(
  logical_device: &ash::Device,
  id: GraphicsPipelineHandle,
  cache: vk::PipelineCache,
) {
  let pipeline_cache_data = logical_device.get_pipeline_cache_data(cache).unwrap();
  std::fs::write(format!("cache/{}.bin", id.0), pipeline_cache_data).unwrap();
  logical_device.destroy_pipeline_cache(cache, None);
}

pub struct PipelineManager {
  max_graphics_id: u64,
  graphics_pipelines: HashMap<GraphicsPipelineHandle, GraphicsPipeline>,
  light_pipeline: Option<GraphicsPipeline>,
  logical_device: ash::Device,
  render_pass: vk::RenderPass,
  swapchain_extent: vk::Extent2D,
  graphics_changed: bool,
}

impl PipelineManager {
  #[inline]
  pub(crate) fn init(
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain: &SwapChain,
  ) -> Self {
    Self {
      max_graphics_id: 0,
      graphics_pipelines: HashMap::new(),
      light_pipeline: None,
      logical_device: logical_device.clone(),
      render_pass,
      swapchain_extent: swapchain.get_extent(),
      graphics_changed: false,
    }
  }

  pub fn build_graphics_pipeline(
    &mut self,
    builder: GraphicsPipelineBuilder<'_>,
    descriptor_manager: &DescriptorManager,
  ) -> Option<GraphicsPipelineHandle> {
    let id = GraphicsPipelineHandle(self.max_graphics_id);
    self.max_graphics_id += 1;

    let subpass = builder.rendering_stage.subpass();
    let is_light = builder.rendering_stage == RenderingStage::Light;

    let pipeline = builder
      .build(
        &self.logical_device,
        descriptor_manager,
        self.render_pass,
        self.swapchain_extent,
        id,
        subpass,
      )
      .ok()?;

    if is_light {
      self.light_pipeline = Some(pipeline);
    } else {
      self.graphics_pipelines.insert(id, pipeline);
    }
    self.graphics_changed = true;

    Some(id)
  }

  pub(crate) fn cleanup(&self) {
    std::fs::create_dir_all("cache").unwrap();
    for pipeline in self.graphics_pipelines.values() {
      pipeline.cleanup(&self.logical_device);
    }
    self
      .light_pipeline
      .as_ref()
      .unwrap()
      .cleanup(&self.logical_device);
  }

  #[inline]
  pub(crate) fn light_pipeline(&self) -> &GraphicsPipeline {
    self.light_pipeline.as_ref().unwrap()
  }

  #[inline]
  pub(crate) fn graphics_pipelines(&self) -> Vec<&GraphicsPipeline> {
    self.graphics_pipelines.values().collect()
  }

  #[inline]
  pub(crate) fn graphics_changed(&self) -> bool {
    self.graphics_changed
  }

  #[inline]
  pub(crate) fn graphics_changed_reset(&mut self) {
    self.graphics_changed = false;
  }
}
