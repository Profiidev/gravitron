use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use crate::config::vulkan::{ImageConfig, ImageData};

use super::{buffer::Buffer, image::Image, manager::Transfer};

pub struct SamplerImage {
  image: Image,
  sampler: vk::Sampler,
}

impl SamplerImage {
  pub fn new(
    image_config: &ImageConfig,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Result<Self, Error> {
    let image_file = match &image_config.data {
      ImageData::Path(path) => image::open(path)?.to_rgba8(),
      ImageData::Bytes(bytes) => image::load_from_memory(bytes)?.to_rgba8(),
    };
    let (width, height) = image_file.dimensions();

    let image_info = vk::ImageCreateInfo::default()
      .image_type(vk::ImageType::TYPE_2D)
      .format(vk::Format::R8G8B8A8_SRGB)
      .extent(vk::Extent3D {
        width,
        height,
        depth: 1,
      })
      .mip_levels(1)
      .array_layers(1)
      .samples(vk::SampleCountFlags::TYPE_1)
      .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST);

    let subresource_range = vk::ImageSubresourceRange {
      aspect_mask: vk::ImageAspectFlags::COLOR,
      layer_count: 1,
      level_count: 1,
      ..Default::default()
    };

    let image_view_info = vk::ImageViewCreateInfo::default()
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(vk::Format::R8G8B8A8_SRGB)
      .subresource_range(subresource_range);

    let sampler_info = vk::SamplerCreateInfo::default()
      .mag_filter(image_config.interpolation)
      .min_filter(image_config.interpolation);

    let image = Image::new(
      device,
      allocator,
      gpu_allocator::MemoryLocation::GpuOnly,
      &image_info,
      &image_view_info,
    )?;
    let sampler = unsafe { device.create_sampler(&sampler_info, None)? };

    let data = image_file.into_raw();
    let mut transfer_buffer = Buffer::new(
      allocator,
      device,
      data.len(),
      vk::BufferUsageFlags::TRANSFER_SRC,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;
    transfer_buffer.fill(&data)?;

    let begin_info =
      vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    let image_subresource = vk::ImageSubresourceLayers::default()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .layer_count(1)
      .base_array_layer(0)
      .mip_level(0);
    let region = vk::BufferImageCopy::default()
      .image_subresource(image_subresource)
      .image_extent(vk::Extent3D {
        width,
        height,
        depth: 1,
      });

    let transfer_barrier = vk::ImageMemoryBarrier::default()
      .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
      .old_layout(vk::ImageLayout::UNDEFINED)
      .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
      .image(image.image())
      .subresource_range(subresource_range);

    let layout_barrier = vk::ImageMemoryBarrier::default()
      .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
      .dst_access_mask(vk::AccessFlags::SHADER_READ)
      .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
      .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
      .image(image.image())
      .subresource_range(subresource_range);

    let command_buffer = transfer.buffer();
    let command_buffers = [command_buffer];

    let submit_info = [vk::SubmitInfo::default().command_buffers(&command_buffers)];

    unsafe {
      device.begin_command_buffer(command_buffer, &begin_info)?;
      device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[transfer_barrier],
      );
      device.cmd_copy_buffer_to_image(
        command_buffer,
        transfer_buffer.buffer(),
        image.image(),
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
      );
      device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[layout_barrier],
      );
      device.end_command_buffer(command_buffer)?;

      device.reset_fences(&[transfer.fence()])?;
      device.queue_submit(transfer.queue(), &submit_info, transfer.fence())?;
      device.wait_for_fences(&[transfer.fence()], true, u64::MAX)?;

      transfer_buffer.cleanup(device, allocator)?;
    }

    Ok(Self { image, sampler })
  }

  pub fn cleanup(
    self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    unsafe {
      device.destroy_sampler(self.sampler, None);
    }
    self.image.cleanup(device, allocator)
  }

  pub fn image_view(&self) -> vk::ImageView {
    self.image.image_view()
  }

  pub fn sampler(&self) -> vk::Sampler {
    self.sampler
  }
}
