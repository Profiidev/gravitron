use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;
use pipeline::PipelineManager;
use pools::Pools;
use swap_chain::SwapChain;

use crate::config::{app::AppConfig, vulkan::VulkanConfig};

use super::{device::Device, error::RendererInitError, instance::InstanceDevice, surface::Surface};

mod pipeline;
mod pools;
mod swap_chain;

pub struct Renderer {
  render_pass: ash::vk::RenderPass,
  swap_chain: SwapChain,
  pipeline: PipelineManager,
  pools: Pools,
}

impl Renderer {
  pub fn init(
    instance: &InstanceDevice,
    device: &Device,
    allocator: &mut vulkan::Allocator,
    surface: &Surface,
    config: &mut VulkanConfig,
    app_config: &AppConfig,
  ) -> Result<Self, Error> {
    let mut pools = Pools::init(device.get_device(), device.get_queue_families())?;

    let format = surface
      .get_formats(instance.get_physical_device())?
      .first()
      .ok_or(RendererInitError::FormatMissing)?
      .format;
    let render_pass = pipeline::init_render_pass(device.get_device(), format)?;
    let swap_chain = SwapChain::init(
      instance,
      device,
      surface,
      allocator,
      app_config,
      &mut pools,
      render_pass,
    )?;
    let pipeline = PipelineManager::init(
      device.get_device(),
      render_pass,
      &swap_chain.get_extent(),
      &mut config.shaders,
    )?;

    Ok(Self {
      render_pass,
      swap_chain,
      pipeline,
      pools,
    })
  }

  pub fn destroy(&mut self, logical_device: &ash::Device, allocator: &mut vulkan::Allocator) {
    unsafe {
      self.pools.cleanup(logical_device);
    }
    self.pipeline.destroy(logical_device);
    unsafe {
      logical_device.destroy_render_pass(self.render_pass, None);
    }
    self.swap_chain.destroy(logical_device, allocator);
  }

  pub fn get_swapchain(&self) -> &SwapChain {
    &self.swap_chain
  }

  pub fn get_swapchain_mut(&mut self) -> &mut SwapChain {
    &mut self.swap_chain
  }

  pub fn testing(&self, device: &ash::Device) -> Result<(), vk::Result> {
    self.swap_chain.testing(device, self.render_pass)
  }
}
