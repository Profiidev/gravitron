use std::{collections::HashMap, ffi::CStr};

use anyhow::Error;
use ash::vk;

use crate::renderer::swapchain::SwapChain;

use super::{
  graphics::{stage::RenderingStage, GraphicsPipeline, GraphicsPipelineBuilder},
  DescriptorManager,
};

pub(crate) const MAIN_FN: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct GraphicsPipelineId(u64);

pub(crate) fn create_pipeline_cache(
  logical_device: &ash::Device,
  id: GraphicsPipelineId,
) -> Result<vk::PipelineCache, Error> {
  let initial_data = std::fs::read(format!("cache/{}.bin", id.0)).unwrap_or_default();

  let pipeline_cache_create_info =
    vk::PipelineCacheCreateInfo::default().initial_data(&initial_data);

  Ok(unsafe { logical_device.create_pipeline_cache(&pipeline_cache_create_info, None) }?)
}

pub(crate) unsafe fn cleanup_pipeline_cache(
  logical_device: &ash::Device,
  id: GraphicsPipelineId,
  cache: vk::PipelineCache,
) {
  let pipeline_cache_data = logical_device.get_pipeline_cache_data(cache).unwrap();
  std::fs::write(format!("cache/{}.bin", id.0), pipeline_cache_data).unwrap();
  logical_device.destroy_pipeline_cache(cache, None);
}

pub struct PipelineManager {
  max_graphics_id: u64,
  graphics_pipelines: HashMap<GraphicsPipelineId, GraphicsPipeline>,
  light_pipeline: Option<GraphicsPipeline>,
  logical_device: ash::Device,
  render_pass: vk::RenderPass,
  swapchain_extent: vk::Extent2D,
}

impl PipelineManager {
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
    }
  }

  pub fn build_graphics_pipeline(
    &mut self,
    builder: GraphicsPipelineBuilder<'_>,
    descriptor_manager: &DescriptorManager,
  ) -> Option<GraphicsPipelineId> {
    let id = GraphicsPipelineId(self.max_graphics_id);
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

  pub(crate) fn graphics_pipeline(&self, id: GraphicsPipelineId) -> Option<&GraphicsPipeline> {
    self.graphics_pipelines.get(&id)
  }

  pub(crate) fn light_pipeline(&self) -> &GraphicsPipeline {
    &self.light_pipeline.as_ref().unwrap()
  }

  pub(crate) fn graphics_pipelines(&self) -> Vec<&GraphicsPipeline> {
    self.graphics_pipelines.values().collect()
  }
}
