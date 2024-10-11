use std::collections::HashMap;

use anyhow::Error;
use ash::vk;
use resources::model::{ModelId, ModelManager};
use swap_chain::SwapChain;

use crate::config::{app::AppConfig, vulkan::VulkanConfig};

use super::{
  device::Device,
  error::RendererInitError,
  instance::InstanceDevice,
  memory::{
    manager::{BufferBlockSize, BufferId, MemoryManager},
    BufferMemory,
  },
  pipeline::{self, pools::Pools, PipelineManager},
  surface::Surface,
};

pub mod resources;
mod swap_chain;

pub struct Renderer {
  render_pass: ash::vk::RenderPass,
  swap_chain: SwapChain,
  model_manager: ModelManager,
  logical_device: ash::Device,
  draw_commands: BufferId,
  draw_commands_mem: Option<BufferMemory>,
  draw_count: BufferId,
  draw_count_mem: BufferMemory,
  draw_count_data: u32,
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
    let render_pass = pipeline::init_render_pass(logical_device, format, config.shaders.len() + 1)?;
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
      BufferBlockSize::Exact(4),
    )?;
    let draw_commands_mem = memory_manager.reserve_buffer_mem(draw_commands, 1).unwrap();
    let draw_count_mem = memory_manager.reserve_buffer_mem(draw_count, 4).unwrap();

    Ok(Self {
      render_pass,
      swap_chain,
      model_manager,
      logical_device: logical_device.clone(),
      draw_commands,
      draw_commands_mem: Some(draw_commands_mem),
      draw_count,
      draw_count_mem,
      draw_count_data: 0,
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
        let max_draw_count = memory_manager.get_buffer_size(self.draw_commands).unwrap() / 20;
        self.logical_device.cmd_draw_indexed_indirect_count(
          buffer,
          draw_commands,
          0,
          draw_count,
          0,
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

    let cmd_new_length = cmd_new.len();

    let cmd_size = std::mem::size_of::<vk::DrawIndexedIndirectCommand>();
    let size_needed = cmd_size * (self.draw_count_data as usize + cmd_new.len());
    let old_mem = std::mem::take(&mut self.draw_commands_mem).unwrap();
    memory_manager.free_buffer_mem(old_mem);
    self.draw_commands_mem = Some(
      memory_manager
        .reserve_buffer_mem(self.draw_commands, size_needed)
        .unwrap(),
    );

    let mut to_write = Vec::new();
    for (i, (model_id, shader, cmd)) in cmd_new.into_iter().enumerate() {
      let model = self.commands.entry(model_id).or_default();
      model.insert(
        shader,
        (
          cmd,
          (self.draw_count_data as u64 + i as u64) * cmd_size as u64,
        ),
      );
      to_write.push(cmd);
    }

    let copy_info = [vk::BufferCopy {
      src_offset: 0,
      dst_offset: self.draw_count_data as u64 * cmd_size as u64,
      size: (to_write.len() * cmd_size) as u64,
    }];
    let to_write_slice = to_write.as_slice();
    memory_manager.write_to_buffer_direct(self.draw_commands, to_write_slice, &copy_info);

    self.draw_count_data += cmd_new_length as u32;
    memory_manager.write_to_buffer(&self.draw_count_mem, &[self.draw_count_data]);
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
