use anyhow::Error;
use ash::vk;
use stage::RenderingStage;

use crate::pipeline::manager::MAIN_FN;

pub mod stage;

use super::{
  descriptor::{manager::DescriptorManager, DescriptorSetId},
  manager::{cleanup_pipeline_cache, create_pipeline_cache, GraphicsPipelineId},
};

#[derive(Default)]
pub struct GraphicsPipelineBuilder<'s> {
  pub vertex_shader: Option<&'s [u32]>,
  pub geometry_shader: Option<&'s [u32]>,
  pub fragment_shader: Option<&'s [u32]>,
  pub descriptor_sets: Vec<DescriptorSetId>,
  pub rendering_stage: RenderingStage,
}

impl<'s> GraphicsPipelineBuilder<'s> {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn vertex_shader(mut self, shader: &'s [u32]) -> Self {
    self.vertex_shader = Some(shader);
    self
  }

  #[inline]
  pub fn geometry_shader(mut self, shader: &'s [u32]) -> Self {
    self.geometry_shader = Some(shader);
    self
  }

  #[inline]
  pub fn fragment_shader(mut self, shader: &'s [u32]) -> Self {
    self.fragment_shader = Some(shader);
    self
  }

  #[inline]
  pub fn add_descriptor_sets(mut self, sets: Vec<DescriptorSetId>) -> Self {
    self.descriptor_sets.extend(sets);
    self
  }

  #[inline]
  pub fn add_descriptor_set(mut self, set: DescriptorSetId) -> Self {
    self.descriptor_sets.push(set);
    self
  }

  #[inline]
  pub fn rendering_stage(mut self, stage: RenderingStage) -> Self {
    self.rendering_stage = stage;
    self
  }

  pub(crate) fn build(
    self,
    logical_device: &ash::Device,
    descriptor_manager: &DescriptorManager,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    id: GraphicsPipelineId,
    subpass: u32,
  ) -> Result<GraphicsPipeline, Error> {
    //Shader code
    let mut modules = Vec::new();

    let shader_create_info = vk::ShaderModuleCreateInfo::default().code(
      self
        .vertex_shader
        .unwrap_or(self.rendering_stage.vertex_shader()),
    );
    let shader_module = unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
    modules.push((shader_module, vk::ShaderStageFlags::VERTEX));

    if let Some(code) = self.geometry_shader {
      let shader_create_info = vk::ShaderModuleCreateInfo::default().code(code);
      let shader_module =
        unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
      modules.push((shader_module, vk::ShaderStageFlags::GEOMETRY));
    }

    let shader_create_info = vk::ShaderModuleCreateInfo::default().code(
      self
        .fragment_shader
        .unwrap_or(self.rendering_stage.fragment_shader()),
    );
    let shader_module = unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
    modules.push((shader_module, vk::ShaderStageFlags::FRAGMENT));

    let mut stages = Vec::new();
    for (module, stage) in &modules {
      let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(*stage)
        .module(*module)
        .name(MAIN_FN);
      stages.push(stage);
    }

    //Inputs
    let (vertex_binding, vertex_attrib) = self.rendering_stage.inputs();

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
      .vertex_binding_descriptions(&vertex_binding)
      .vertex_attribute_descriptions(&vertex_attrib);
    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
      .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    //Viewport
    let viewport_size = (swapchain_extent.width, swapchain_extent.height);

    let viewport = [vk::Viewport::default()
      .x(0.0)
      .y(0.0)
      .width(viewport_size.0 as f32)
      .height(viewport_size.1 as f32)
      .min_depth(0.0)
      .max_depth(1.0)];
    let scissor = [vk::Rect2D::default()
      .offset(vk::Offset2D::default())
      .extent(vk::Extent2D {
        width: viewport_size.0,
        height: viewport_size.1,
      })];

    let viewport_info = vk::PipelineViewportStateCreateInfo::default()
      .viewports(&viewport)
      .scissors(&scissor);

    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
      .line_width(1.0)
      .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
      .cull_mode(vk::CullModeFlags::BACK)
      .polygon_mode(vk::PolygonMode::FILL);

    let multisample_info = vk::PipelineMultisampleStateCreateInfo::default()
      .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    //Output
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
      .color_write_mask(
        vk::ColorComponentFlags::R
          | vk::ColorComponentFlags::G
          | vk::ColorComponentFlags::B
          | vk::ColorComponentFlags::A,
      )
      .blend_enable(false)
      .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .color_blend_op(vk::BlendOp::ADD)
      .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .alpha_blend_op(vk::BlendOp::ADD);
    let color_blend_attachments = self.rendering_stage.output(color_blend_attachment);

    let color_blend_info =
      vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

    let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
      .depth_test_enable(true)
      .depth_write_enable(true)
      .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

    //Layout
    let layouts = descriptor_manager.vk_layouts(&self.descriptor_sets);
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    let layout =
      unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;

    //General
    let pipeline_create_info = self.rendering_stage.depth_buffer(
      vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly_info)
        .viewport_state(&viewport_info)
        .rasterization_state(&rasterizer_info)
        .multisample_state(&multisample_info)
        .color_blend_state(&color_blend_info)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(subpass),
      &depth_stencil_info,
    );

    //Cache
    let cache = create_pipeline_cache(logical_device, id)?;

    //Creation
    let pipeline = unsafe {
      logical_device
        .create_graphics_pipelines(cache, &[pipeline_create_info], None)
        .expect("Unable to create graphics pipeline")
    }[0];

    for (module, _) in modules {
      unsafe {
        logical_device.destroy_shader_module(module, None);
      }
    }

    Ok(GraphicsPipeline {
      id,
      pipeline,
      layout,
      descriptor_sets: self.descriptor_sets,
      cache,
    })
  }
}

pub(crate) struct GraphicsPipeline {
  id: GraphicsPipelineId,
  pipeline: vk::Pipeline,
  layout: vk::PipelineLayout,
  descriptor_sets: Vec<DescriptorSetId>,
  cache: vk::PipelineCache,
}

impl GraphicsPipeline {
  pub fn cleanup(&self, logical_device: &ash::Device) {
    unsafe {
      logical_device.destroy_pipeline(self.pipeline, None);
      logical_device.destroy_pipeline_layout(self.layout, None);
      cleanup_pipeline_cache(logical_device, self.id, self.cache);
    }
  }

  #[inline]
  pub unsafe fn bind(
    &self,
    command_buffer: vk::CommandBuffer,
    logical_device: &ash::Device,
    descriptor_manager: &DescriptorManager,
  ) {
    logical_device.cmd_bind_pipeline(
      command_buffer,
      vk::PipelineBindPoint::GRAPHICS,
      self.pipeline,
    );

    let sets = descriptor_manager.vk_sets(&self.descriptor_sets);
    logical_device.cmd_bind_descriptor_sets(
      command_buffer,
      vk::PipelineBindPoint::GRAPHICS,
      self.layout,
      0,
      &sets,
      &[],
    );
  }

  #[inline]
  pub fn id(&self) -> GraphicsPipelineId {
    self.id
  }
}
