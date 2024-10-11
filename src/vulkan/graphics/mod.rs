use std::collections::HashMap;

use anyhow::Error;
use ash::vk;
use resources::model::{ModelId, ModelManager};
use swap_chain::SwapChain;

use crate::config::{app::AppConfig, vulkan::{PipelineType, VulkanConfig}};

use super::{
  device::Device,
  error::RendererInitError,
  instance::InstanceDevice,
  memory::{
    manager::{BufferBlockSize, BufferId, MemoryManager},
    BufferMemory,
  },
  pipeline::{pools::Pools, PipelineManager},
  surface::Surface,
};

pub mod resources;
mod swap_chain;
mod render_pass;

pub struct Renderer {
  render_pass: ash::vk::RenderPass,
  swap_chain: SwapChain,
  model_manager: ModelManager,
  logical_device: ash::Device,
  draw_commands: BufferId,
  draw_count: BufferId,
  shader_mem: HashMap<String, (BufferMemory, BufferMemory, u32)>,
  commands: HashMap<ModelId, HashMap<String, (vk::DrawIndexedIndirectCommand, u64)>>,
  buffers_updated: Vec<usize>,
}

impl Renderer {
  pub fn init(
    instance: &InstanceDevice,
    device: &Device,
    memory_manager: &mut MemoryManager,
    surface: &Surface,
    config: &mut VulkanConfig,
    app_config: &AppConfig,
    pools: &mut Pools,
  ) -> Result<Self, Error> {
    let logical_device = device.get_device();

    let format = surface
      .get_formats(instance.get_physical_device())?
      .first()
      .ok_or(RendererInitError::FormatMissing)?
      .format;
    let render_pass = render_pass::init_render_pass(logical_device, format, config.shaders.len() + 1)?;
    let swap_chain = SwapChain::init(
      instance,
      device,
      surface,
      memory_manager,
      app_config,
      pools,
      render_pass,
    )?;

    let model_manager = ModelManager::new(memory_manager)?;

    let draw_commands = memory_manager.create_buffer(
      vk::BufferUsageFlags::INDIRECT_BUFFER,
      BufferBlockSize::Medium,
    )?;
    let draw_count = memory_manager.create_buffer(
      vk::BufferUsageFlags::INDIRECT_BUFFER,
      BufferBlockSize::Small,
    )?;

    let cmd_block_size = 10 * std::mem::size_of::<vk::DrawIndexedIndirectCommand>();
    let mut shader_mem = HashMap::new();

    for pipeline in &config.shaders {
      if let PipelineType::Graphics(shader) = pipeline {
        let cmd_mem = memory_manager.reserve_buffer_mem(draw_commands, cmd_block_size).unwrap();
        let count_mme = memory_manager.reserve_buffer_mem(draw_count, 4).unwrap();
        shader_mem.insert(shader.name.clone(), (cmd_mem, count_mme, 0));
      }
    }

    let cmd_mem = memory_manager.reserve_buffer_mem(draw_commands, cmd_block_size).unwrap();
    let count_mme = memory_manager.reserve_buffer_mem(draw_count, 4).unwrap();
    shader_mem.insert("default".into(), (cmd_mem, count_mme, 0));

    Ok(Self {
      render_pass,
      swap_chain,
      model_manager,
      logical_device: logical_device.clone(),
      draw_commands,
      shader_mem,
      draw_count,
      commands: HashMap::new(),
      buffers_updated: Vec::new(),
    })
  }

  pub fn destroy(&mut self) {
    unsafe {
      self
        .logical_device
        .destroy_render_pass(self.render_pass, None);
    }
    self.swap_chain.destroy(&self.logical_device);
  }

  pub fn wait_for_draw_start(&self, logical_device: &ash::Device) {
    self.swap_chain.wait_for_draw_start(logical_device);
  }

  pub fn record_command_buffer(
    &mut self,
    pipeline_manager: &PipelineManager,
    memory_manager: &mut MemoryManager,
  ) -> Result<(), vk::Result> {
    let buffer_changed = memory_manager.buffer_resize();
    if !buffer_changed
      && self
        .buffers_updated
        .contains(&self.swap_chain.current_frame())
    {
      return Ok(());
    }

    if buffer_changed {
      self.buffers_updated = Vec::new();
      memory_manager.buffer_resize_reset();
    }

    let buffer = self
      .swap_chain
      .record_command_buffer_first(&self.logical_device, self.render_pass)?;

    let names = pipeline_manager.pipeline_names();
    let pipeline_count = names.len();

    self
      .model_manager
      .record_command_buffer(memory_manager, buffer, &self.logical_device);

    for (i, pipeline) in names.into_iter().enumerate() {
      unsafe {
        pipeline_manager
          .get_pipeline(pipeline)
          .unwrap()
          .record_command_buffer(buffer, &self.logical_device);

        let draw_commands = memory_manager.get_vk_buffer(self.draw_commands).unwrap();
        let draw_count = memory_manager.get_vk_buffer(self.draw_count).unwrap();
        let (cmd_mem, count_mem, _) = self.shader_mem.get(pipeline).unwrap();
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

      if i + 1 < pipeline_count {
        unsafe {
          self
            .logical_device
            .cmd_next_subpass(buffer, vk::SubpassContents::INLINE);
        }
      }
    }

    self
      .swap_chain
      .record_command_buffer_second(&self.logical_device, buffer)?;

    self.buffers_updated.push(self.swap_chain.current_frame());
    Ok(())
  }

  pub fn update_draw_buffer(
    &mut self,
    memory_manager: &mut MemoryManager,
    instances: HashMap<ModelId, HashMap<String, Vec<resources::model::InstanceData>>>,
  ) {
    let cmd_new = self.model_manager.update_draw_buffer(
      self.draw_commands,
      &mut self.commands,
      memory_manager,
      instances,
    );

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
        let new_size = (required_size as f32 / cmd_block_size as f32).ceil() as usize * cmd_block_size;
        memory_manager.resize_buffer_mem(cmd_mem, new_size);
        self.buffers_updated = Vec::new();
      }

      write_info.push(vk::BufferCopy {
        src_offset: (write_data.len() * cmd_size) as u64,
        dst_offset: (cmd_mem.offset() + *count as usize * cmd_size) as u64,
        size: (cmd_size * cmd_new_len) as u64,
      });

      for (i, (model_id, cmd)) in cmd_new.into_iter().enumerate() {
        let model = self.commands.entry(model_id).or_default();
        model.insert(shader.clone(), (cmd, (cmd_mem.offset() + (*count as usize + i) * cmd_size) as u64));
        write_data.push(cmd);
      }

      *count += cmd_new_len as u32;
      memory_manager.write_to_buffer(count_mem, &[*count]);
    }

    let write_data_slice = write_data.as_slice();
    memory_manager.write_to_buffer_direct(self.draw_commands, write_data_slice, &write_info);
  }

  pub fn draw_frame(&mut self, device: &Device) {
    self.swap_chain.draw_frame(device);
  }

  pub fn render_pass(&self) -> vk::RenderPass {
    self.render_pass
  }

  pub fn swapchain(&self) -> &SwapChain {
    &self.swap_chain
  }
}
