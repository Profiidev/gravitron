use anyhow::Error;
use ash::{khr, vk};
use gravitron_plugin::config::window::WindowConfig;

use crate::{
  device::Device,
  instance::InstanceDevice,
  memory::{types::ImageId, MemoryManager},
  pipeline::pools::{CommandBufferType, Pools},
  surface::Surface,
};

use super::framebuffer::{Framebuffer, IMAGES_PER_FRAME_BUFFER};

pub struct SwapChain {
  loader: khr::swapchain::Device,
  swapchain: vk::SwapchainKHR,
  framebuffers: Vec<Framebuffer>,
  extent: vk::Extent2D,
  current_image: usize,
  graphics_queue: vk::Queue,
  attachments: [ImageId; IMAGES_PER_FRAME_BUFFER as usize],
}

impl SwapChain {
  pub fn init(
    instance_device: &InstanceDevice,
    device: &Device,
    surfaces: &Surface,
    memory_manager: &mut MemoryManager,
    window_config: &WindowConfig,
    pools: &mut Pools,
    render_pass: vk::RenderPass,
  ) -> Result<Self, Error> {
    let physical_device = instance_device.get_physical_device();
    let logical_device = device.get_device();

    let surface_capabilities = surfaces.get_capabilities(physical_device)?;
    let surface_format = *surfaces.get_formats(physical_device)?.first().unwrap();

    let mut extent = surface_capabilities.current_extent;
    if extent.width == u32::MAX || extent.height == u32::MAX {
      extent.width = window_config.width;
      extent.height = window_config.height;
    }

    let frame_buffer_count =
      if surface_capabilities.min_image_count <= surface_capabilities.max_image_count {
        3.max(surface_capabilities.min_image_count)
          .min(surface_capabilities.max_image_count)
      } else {
        if surface_capabilities.min_image_count < 1 {
          panic!("Inconsistent possible Swapchain ImageCounts");
        }
        surface_capabilities.min_image_count
      };

    let present_mode = if surfaces
      .get_present_modes(physical_device)?
      .contains(&vk::PresentModeKHR::MAILBOX)
    {
      vk::PresentModeKHR::MAILBOX
    } else {
      vk::PresentModeKHR::FIFO
    };

    let queue_families = [device.get_queue_families().get_graphics_q_index()];
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
      .surface(surfaces.get_surface())
      .min_image_count(frame_buffer_count)
      .image_format(surface_format.format)
      .image_color_space(surface_format.color_space)
      .image_extent(extent)
      .image_array_layers(1)
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
      .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
      .queue_family_indices(&queue_families)
      .pre_transform(surface_capabilities.current_transform)
      .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
      .present_mode(present_mode);

    let swapchain_loader =
      khr::swapchain::Device::new(instance_device.get_instance(), logical_device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;

    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

    let extend_3d = vk::Extent3D {
      width: extent.width,
      height: extent.height,
      depth: 1,
    };
    let depth_image_create_info = vk::ImageCreateInfo::default()
      .image_type(vk::ImageType::TYPE_2D)
      .format(vk::Format::D32_SFLOAT)
      .extent(extend_3d)
      .mip_levels(1)
      .array_layers(1)
      .samples(vk::SampleCountFlags::TYPE_1)
      .tiling(vk::ImageTiling::OPTIMAL)
      .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
      .sharing_mode(vk::SharingMode::EXCLUSIVE)
      .queue_family_indices(&queue_families);

    let subresource_range = vk::ImageSubresourceRange::default()
      .aspect_mask(vk::ImageAspectFlags::DEPTH)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1);
    let depth_image_view_create_info = vk::ImageViewCreateInfo::default()
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(vk::Format::D32_SFLOAT)
      .subresource_range(subresource_range);

    let depth_image =
      memory_manager.create_image(&depth_image_create_info, &depth_image_view_create_info)?;

    let command_buffer = pools.create_command_buffers(
      logical_device,
      swapchain_images.len(),
      CommandBufferType::Graphics,
    )?;

    let image_info = depth_image_create_info
      .usage(vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT)
      .format(vk::Format::R32G32B32A32_SFLOAT);

    let subresource_range = subresource_range.aspect_mask(vk::ImageAspectFlags::COLOR);
    let image_view_info = depth_image_view_create_info
      .format(vk::Format::R32G32B32A32_SFLOAT)
      .subresource_range(subresource_range);

    let mut images = Vec::new();
    for _ in 0..IMAGES_PER_FRAME_BUFFER {
      images.push(memory_manager.create_image(&image_info, &image_view_info)?);
    }
    let images = [images[0], images[1], images[2]];

    let mut framebuffers = Vec::new();
    for (swapchain_image, command_buffer) in swapchain_images.into_iter().zip(command_buffer) {
      framebuffers.push(Framebuffer::create(
        swapchain_image,
        logical_device,
        surface_format.format,
        images,
        depth_image,
        render_pass,
        memory_manager,
        extent,
        command_buffer,
      )?);
    }

    Ok(Self {
      loader: swapchain_loader,
      swapchain,
      framebuffers,
      extent,
      current_image: 0,
      graphics_queue: device.get_queues().graphics(),
      attachments: images,
    })
  }

  #[inline]
  pub fn get_extent(&self) -> vk::Extent2D {
    self.extent
  }

  #[inline]
  pub fn cleanup(&self, logical_device: &ash::Device) {
    for framebuffer in &self.framebuffers {
      framebuffer.cleanup(logical_device);
    }

    unsafe {
      self.loader.destroy_swapchain(self.swapchain, None);
    }
  }

  #[inline]
  pub fn wait_for_draw_start(&self, device: &ash::Device) {
    unsafe {
      device
        .wait_for_fences(
          &[self.framebuffers[self.current_image].begin_drawing()],
          true,
          u64::MAX,
        )
        .expect("Unable to wait for fences");

      device
        .reset_fences(&[self.framebuffers[self.current_image].begin_drawing()])
        .expect("Unable to reset Fence");
    }
  }

  #[inline]
  pub fn record_command_buffer_start(
    &self,
    device: &ash::Device,
    render_pass: vk::RenderPass,
  ) -> Result<vk::CommandBuffer, vk::Result> {
    self.framebuffers[self.current_image].start_record(device, render_pass, self.extent)
  }

  #[inline]
  pub fn record_command_buffer_end(
    &self,
    device: &ash::Device,
    buffer: vk::CommandBuffer,
  ) -> Result<(), vk::Result> {
    unsafe {
      device.cmd_end_render_pass(buffer);
      device.end_command_buffer(buffer)
    }
  }

  pub fn draw_frame(&mut self, logical_device: &ash::Device) {
    let (image_index, _) = unsafe {
      self
        .loader
        .acquire_next_image(
          self.swapchain,
          u64::MAX,
          self.framebuffers[self.current_image].available(),
          vk::Fence::null(),
        )
        .expect("Unable to acquire next image")
    };

    let semaphore_available = [self.framebuffers[self.current_image].available()];
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let semaphore_render_finished = [self.framebuffers[self.current_image].finished()];
    let command_buffer = [self.framebuffers[self.current_image].command_buffer()];

    let submit_info = [vk::SubmitInfo::default()
      .wait_semaphores(&semaphore_available)
      .wait_dst_stage_mask(&wait_stages)
      .command_buffers(&command_buffer)
      .signal_semaphores(&semaphore_render_finished)];

    unsafe {
      logical_device
        .queue_submit(
          self.graphics_queue,
          &submit_info,
          self.framebuffers[self.current_image].begin_drawing(),
        )
        .expect("Unable to submit queue");
    }

    let swapchains = [self.swapchain];
    let image_indices = [image_index];
    let present_info = vk::PresentInfoKHR::default()
      .wait_semaphores(&semaphore_render_finished)
      .swapchains(&swapchains)
      .image_indices(&image_indices);
    unsafe {
      self
        .loader
        .queue_present(self.graphics_queue, &present_info)
        .expect("Unable to queue present");
    }

    self.current_image = (self.current_image + 1) % self.framebuffers.len();
  }

  #[inline]
  pub fn current_frame(&self) -> usize {
    self.current_image
  }

  #[inline]
  pub fn attachments(&self) -> &[ImageId] {
    &self.attachments
  }
}
