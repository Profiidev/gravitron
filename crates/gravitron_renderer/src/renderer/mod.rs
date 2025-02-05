use std::collections::HashMap;

use anyhow::Error;
use ash::vk;
use glam::Mat4;
use gravitron_plugin::config::window::WindowConfig;
use resources::lighting::{LightInfo, PointLight, SpotLight};
use swapchain::SwapChain;

use crate::{
  memory::{
    types::{BufferMemory, BufferMemoryLocation},
    MemoryManager,
  },
  model::{
    model::{InstanceData, ModelId},
    ModelManager,
  },
  pipeline::{
    descriptor::{DescriptorId, DescriptorInfo, DescriptorSetId, DescriptorType},
    graphics::{stage::RenderingStage, GraphicsPipelineBuilder},
    manager::GraphicsPipelineId,
    DescriptorManager,
  },
};

use super::{
  device::Device,
  error::RendererInitError,
  instance::InstanceDevice,
  memory::types::{BufferBlockSize, BufferId},
  pipeline::{manager::PipelineManager, pools::Pools},
  surface::Surface,
};

mod framebuffer;
mod render_pass;
pub mod resources;
pub(crate) mod swapchain;

pub const DEFAULT_DESCRIPTOR_SET: DescriptorSetId = DescriptorSetId(0);
pub const TEXTURE_DESCRIPTOR_SET: DescriptorSetId = DescriptorSetId(1);
pub const ATTACHMENT_DESCRIPTOR_SET: DescriptorSetId = DescriptorSetId(2);

pub const CAMERA_DESCRIPTOR: DescriptorId = DescriptorId(0);
pub const LIGHT_INFO_DESCRIPTOR: DescriptorId = DescriptorId(1);
pub const POINT_LIGHT_DESCRIPTOR: DescriptorId = DescriptorId(2);
pub const SPOT_LIGHT_DESCRIPTOR: DescriptorId = DescriptorId(3);
pub const TEXTURE_DESCRIPTOR: DescriptorId = DescriptorId(4);

pub struct Renderer {
  render_pass: ash::vk::RenderPass,
  swapchain: SwapChain,
  logical_device: ash::Device,
  draw_commands: BufferId,
  draw_count: BufferId,
  commands: HashMap<ModelId, HashMap<GraphicsPipelineId, (vk::DrawIndexedIndirectCommand, u64)>>,
  buffers_updated: Vec<usize>,
  shader_mem: HashMap<GraphicsPipelineId, (BufferMemory, BufferMemory, u32)>,
  descriptor_buffer: BufferId,
}

impl Renderer {
  pub(crate) fn init(
    instance: &InstanceDevice,
    device: &Device,
    memory_manager: &mut MemoryManager,
    descriptor_manager: &mut DescriptorManager,
    surface: &Surface,
    window_config: &WindowConfig,
    pools: &mut Pools,
  ) -> Result<(Self, PipelineManager), Error> {
    let logical_device = device.get_device();

    let format = surface
      .get_formats(instance.get_physical_device())?
      .first()
      .ok_or(RendererInitError::FormatMissing)?
      .format;
    let render_pass = render_pass::init_render_pass(logical_device, format)?;

    let swapchain = SwapChain::init(
      instance,
      device,
      surface,
      memory_manager,
      window_config,
      pools,
      render_pass,
    )?;

    let draw_commands = memory_manager.create_advanced_buffer(
      vk::BufferUsageFlags::INDIRECT_BUFFER,
      BufferBlockSize::Medium,
    )?;
    let draw_count = memory_manager.create_simple_buffer(
      vk::BufferUsageFlags::INDIRECT_BUFFER,
      BufferBlockSize::Small,
      BufferMemoryLocation::CpuToGpu,
    )?;

    let buffer = memory_manager.create_simple_buffer(
      vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
      BufferBlockSize::Medium,
      BufferMemoryLocation::CpuToGpu,
    )?;

    let camera_mem = memory_manager
      .reserve_buffer_mem(buffer, size_of::<Mat4>() * 2)
      .unwrap();
    let default_texture = memory_manager.create_texture_image(
      vk::Filter::NEAREST,
      include_bytes!("../../assets/default.png"),
    )?;
    let light_info_mem = memory_manager
      .reserve_buffer_mem(buffer, size_of::<LightInfo>())
      .unwrap();
    let point_light_mem = memory_manager
      .reserve_buffer_mem(buffer, size_of::<PointLight>() * 10)
      .unwrap();
    let spot_light_mem = memory_manager
      .reserve_buffer_mem(buffer, size_of::<SpotLight>() * 10)
      .unwrap();
    let descriptor = vec![
      DescriptorInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        r#type: DescriptorType::UniformBuffer(camera_mem),
      },
      DescriptorInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::UniformBuffer(light_info_mem),
      },
      DescriptorInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::StorageBuffer(point_light_mem),
      },
      DescriptorInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::StorageBuffer(spot_light_mem),
      },
    ];
    descriptor_manager
      .create_descriptor_set(descriptor, memory_manager)
      .expect("Failed to create default descriptor set");

    let descriptor = vec![DescriptorInfo {
      stage: vk::ShaderStageFlags::FRAGMENT,
      r#type: DescriptorType::Sampler(vec![default_texture]),
    }];
    descriptor_manager
      .create_descriptor_set(descriptor, memory_manager)
      .expect("Failed to create default descriptor set");

    let images = swapchain.attachments();
    let descriptor = vec![
      DescriptorInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::InputAttachment(images[0]),
      },
      DescriptorInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::InputAttachment(images[1]),
      },
      DescriptorInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::InputAttachment(images[2]),
      },
    ];
    descriptor_manager
      .create_descriptor_set(descriptor, memory_manager)
      .expect("Failed to create attachment descriptor set");

    let mut pipeline_manager = PipelineManager::init(logical_device, render_pass, &swapchain);

    let world = GraphicsPipelineBuilder::new()
      .add_descriptor_sets(vec![DEFAULT_DESCRIPTOR_SET, TEXTURE_DESCRIPTOR_SET]);
    pipeline_manager.build_graphics_pipeline(world, descriptor_manager);
    let light = GraphicsPipelineBuilder::new()
      .rendering_stage(RenderingStage::Light)
      .add_descriptor_sets(vec![DEFAULT_DESCRIPTOR_SET, ATTACHMENT_DESCRIPTOR_SET]);
    pipeline_manager.build_graphics_pipeline(light, descriptor_manager);

    Ok((
      Self {
        render_pass,
        swapchain,
        logical_device: logical_device.clone(),
        draw_commands,
        draw_count,
        commands: HashMap::new(),
        buffers_updated: Vec::new(),
        shader_mem: HashMap::new(),
        descriptor_buffer: buffer,
      },
      pipeline_manager,
    ))
  }

  pub(crate) fn cleanup(&self) {
    unsafe {
      self
        .logical_device
        .destroy_render_pass(self.render_pass, None);
    }
    self.swapchain.cleanup(&self.logical_device);
  }

  #[inline]
  pub(crate) fn wait_for_draw_start(&self) {
    self.swapchain.wait_for_draw_start(&self.logical_device);
  }

  pub(crate) fn record_command_buffer(
    &mut self,
    pipeline_manager: &PipelineManager,
    descriptor_manager: &DescriptorManager,
    memory_manager: &mut MemoryManager,
    model_manager: &ModelManager,
  ) -> Result<(), vk::Result> {
    //check for invalidations
    let buffer_ids = [
      self.descriptor_buffer,
      self.draw_commands,
      self.draw_count,
      model_manager.index_buffer_id(),
      model_manager.vertex_buffer_id(),
      model_manager.instance_buffer_id(),
    ];
    if memory_manager.buffer_reallocated(&buffer_ids) || pipeline_manager.graphics_changed() {
      self.buffers_updated.clear();
    }
    self.buffers_updated.clear();

    if self
      .buffers_updated
      .contains(&self.swapchain.current_frame())
    {
      return Ok(());
    }

    let buffer = self
      .swapchain
      .record_command_buffer_start(&self.logical_device, self.render_pass)?;

    model_manager.record_command_buffer(memory_manager, buffer, &self.logical_device);

    for pipeline in pipeline_manager.graphics_pipelines() {
      unsafe {
        pipeline.bind(buffer, &self.logical_device, descriptor_manager);

        let draw_commands = memory_manager.get_vk_buffer(self.draw_commands).unwrap();
        let draw_count = memory_manager.get_vk_buffer(self.draw_count).unwrap();
        let (cmd_mem, count_mem, _) = self.shader_mem.entry(pipeline.id()).or_insert_with(|| {
          (
            memory_manager
              .reserve_buffer_mem(
                self.draw_commands,
                10 * std::mem::size_of::<vk::DrawIndexedIndirectCommand>(),
              )
              .expect("Failed to reserve draw cmd mem"),
            memory_manager
              .reserve_buffer_mem(self.draw_count, 4)
              .expect("Failed to reserve draw cmd mem"),
            0,
          )
        });
        let max_draw_count = cmd_mem.size() / 20;

        self.logical_device.cmd_draw_indexed_indirect_count(
          buffer,
          draw_commands,
          cmd_mem.offset() as u64,
          draw_count,
          count_mem.offset() as u64,
          max_draw_count as u32,
          std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32,
        );
      }
    }

    unsafe {
      self
        .logical_device
        .cmd_next_subpass(buffer, vk::SubpassContents::INLINE);
    }

    unsafe {
      pipeline_manager
        .light_pipeline()
        .bind(buffer, &self.logical_device, descriptor_manager);

      self.logical_device.cmd_draw(buffer, 3, 1, 0, 0);
    }

    self
      .swapchain
      .record_command_buffer_end(&self.logical_device, buffer)?;

    self.buffers_updated.push(self.swapchain.current_frame());
    Ok(())
  }

  pub(crate) fn update_draw_buffer(
    &mut self,
    memory_manager: &mut MemoryManager,
    instances: HashMap<ModelId, HashMap<GraphicsPipelineId, Vec<InstanceData>>>,
    model_manager: &mut ModelManager,
  ) {
    let cmd_new = model_manager
      .update_draw_buffer(
        self.draw_commands,
        &mut self.commands,
        memory_manager,
        instances,
      )
      .unwrap_or_default();

    if cmd_new.is_empty() {
      return;
    }

    let cmd_size = std::mem::size_of::<vk::DrawIndexedIndirectCommand>();
    let cmd_block_size = 10 * cmd_size;

    let mut write_info = Vec::new();
    let mut write_data = Vec::new();

    for (shader, cmd_new) in cmd_new {
      let (cmd_mem, count_mem, count) = self.shader_mem.get_mut(&shader).unwrap();

      let cmd_new_len = cmd_new.len();

      let required_size = cmd_size * (*count as usize + cmd_new_len);
      if cmd_mem.size() < required_size {
        let new_size =
          (required_size as f32 / cmd_block_size as f32).ceil() as usize * cmd_block_size;
        memory_manager.resize_buffer_mem(cmd_mem, new_size).unwrap();
        self.buffers_updated = Vec::new();
      }

      write_info.push(vk::BufferCopy {
        src_offset: (write_data.len() * cmd_size) as u64,
        dst_offset: (cmd_mem.offset() + *count as usize * cmd_size) as u64,
        size: (cmd_size * cmd_new_len) as u64,
      });

      for (i, (model_id, cmd)) in cmd_new.into_iter().enumerate() {
        let model = self.commands.entry(model_id).or_default();
        model.insert(
          shader,
          (
            cmd,
            (cmd_mem.offset() + (*count as usize + i) * cmd_size) as u64,
          ),
        );
        write_data.push(cmd);
      }

      *count += cmd_new_len as u32;
      let _ = memory_manager.write_to_buffer(count_mem, &[*count]);
    }

    let write_data_slice = write_data.as_slice();
    let _ =
      memory_manager.write_to_buffer_direct(self.draw_commands, write_data_slice, &write_info);
  }

  #[inline]
  pub(crate) fn draw_frame(&mut self) {
    self.swapchain.draw_frame(&self.logical_device);
  }
}
