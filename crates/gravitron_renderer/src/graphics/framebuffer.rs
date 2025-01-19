use anyhow::Error;
use ash::vk;

use crate::memory::{manager::MemoryManager, types::ImageId};

pub const IMAGES_PER_FRAME_BUFFER: u32 = 3;

pub struct Framebuffer {
  geometry_buffer: vk::Framebuffer,
  light_buffer: vk::Framebuffer,
  light_view: vk::ImageView,
  available: vk::Semaphore,
  finished: vk::Semaphore,
  begin_drawing: vk::Fence,
  command_buffer: vk::CommandBuffer,
}

impl Framebuffer {
  #[allow(clippy::complexity)]
  pub fn create(
    swapchain_image: vk::Image,
    logical_device: &ash::Device,
    format: vk::Format,
    images: [ImageId; IMAGES_PER_FRAME_BUFFER as usize],
    depth_image: ImageId,
    render_passes: (vk::RenderPass, vk::RenderPass),
    memory_manager: &MemoryManager,
    extent: vk::Extent2D,
    command_buffer: vk::CommandBuffer,
  ) -> Result<Self, Error> {
    let subresource_range = vk::ImageSubresourceRange::default()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1);
    let image_view_create_info = vk::ImageViewCreateInfo::default()
      .image(swapchain_image)
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(format)
      .subresource_range(subresource_range);
    let light_view = unsafe { logical_device.create_image_view(&image_view_create_info, None) }?;

    let vi = [
      memory_manager
        .get_vk_image_view(images[0])
        .expect("Failed to get framebuffer image_view"),
      memory_manager
        .get_vk_image_view(images[1])
        .expect("Failed to get framebuffer image_view"),
      memory_manager
        .get_vk_image_view(images[2])
        .expect("Failed to get framebuffer image_view"),
      memory_manager
        .get_vk_image_view(depth_image)
        .expect("Failed to get framebuffer image_view"),
    ];
    let vi_light = [light_view];

    let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
      .render_pass(render_passes.0)
      .attachments(&vi)
      .width(extent.width)
      .height(extent.height)
      .layers(1);
    let frame_buffer_create_info_light = frame_buffer_create_info
      .render_pass(render_passes.1)
      .attachments(&vi_light);

    let geometry_buffer =
      unsafe { logical_device.create_framebuffer(&frame_buffer_create_info, None) }?;
    let light_buffer =
      unsafe { logical_device.create_framebuffer(&frame_buffer_create_info_light, None) }?;

    let semaphore_create_info = vk::SemaphoreCreateInfo::default();
    let fence_create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    let available = unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }?;
    let finished = unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }?;
    let begin_drawing = unsafe { logical_device.create_fence(&fence_create_info, None) }?;

    Ok(Self {
      geometry_buffer,
      light_buffer,
      light_view,
      available,
      finished,
      begin_drawing,
      command_buffer,
    })
  }

  pub fn destroy(&self, logical_device: &ash::Device) {
    unsafe {
      logical_device.destroy_fence(self.begin_drawing, None);
      logical_device.destroy_semaphore(self.available, None);
      logical_device.destroy_semaphore(self.finished, None);
      logical_device.destroy_framebuffer(self.geometry_buffer, None);
      logical_device.destroy_framebuffer(self.light_buffer, None);
      logical_device.destroy_image_view(self.light_view, None);
    }
  }

  pub fn begin_drawing(&self) -> vk::Fence {
    self.begin_drawing
  }

  pub fn available(&self) -> vk::Semaphore {
    self.available
  }

  pub fn finished(&self) -> vk::Semaphore {
    self.finished
  }

  pub fn command_buffer(&self) -> vk::CommandBuffer {
    self.command_buffer
  }

  pub fn start_record(
    &self,
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
  ) -> Result<vk::CommandBuffer, vk::Result> {
    let buffer_begin_info = vk::CommandBufferBeginInfo::default();
    unsafe {
      device.begin_command_buffer(self.command_buffer, &buffer_begin_info)?;
    }

    let clear_values = [
      vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 1.0],
        },
      },
      vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 0.0],
        },
      },
      vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 0.0],
        },
      },
      vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
          depth: 1.0,
          stencil: 0,
        },
      },
    ];
    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
      .render_pass(render_pass)
      .framebuffer(self.geometry_buffer)
      .render_area(vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent,
      })
      .clear_values(&clear_values);

    unsafe {
      device.cmd_begin_render_pass(
        self.command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
    }

    Ok(self.command_buffer)
  }

  pub fn middle_record(
    &self,
    device: &ash::Device,
    buffer: vk::CommandBuffer,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
  ) {
    unsafe {
      device.cmd_end_render_pass(buffer);
    }

    let clear_values = [vk::ClearValue {
      color: vk::ClearColorValue {
        float32: [0.0, 0.0, 0.0, 1.0],
      },
    }];

    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
      .render_pass(render_pass)
      .framebuffer(self.light_buffer)
      .render_area(vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent,
      })
      .clear_values(&clear_values);

    unsafe {
      device.cmd_begin_render_pass(buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
    }
  }
}
