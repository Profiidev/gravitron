use ash::vk;
use gpu_allocator::vulkan;

use crate::buffer::Buffer;

pub struct Texture {
  pub image: image::RgbaImage,
  pub vk_image: vk::Image,
  pub image_view: vk::ImageView,
  pub sampler: vk::Sampler,
}

impl Texture {
  pub fn from_file<P: AsRef<std::path::Path>>(
    p: P,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    queues: &crate::queues::Queues,
    pools: &crate::pools::Pools,
  ) -> Result<Self, vk::Result> {
    let image = image::open(p).unwrap().to_rgba8();

    let (width, height) = image.dimensions();
    let img_create_info = vk::ImageCreateInfo::default()
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
    let vk_image = unsafe { device.create_image(&img_create_info, None).unwrap() };
    let requirements = unsafe { device.get_image_memory_requirements(vk_image) };

    let alloc_create_desc = vulkan::AllocationCreateDesc {
      requirements,
      location: gpu_allocator::MemoryLocation::GpuOnly,
      name: "Texture",
      linear: false,
      allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
    };

    let allocation = allocator.allocate(&alloc_create_desc).unwrap();
    unsafe {
      device
        .bind_image_memory(vk_image, allocation.memory(), allocation.offset())
        .unwrap()
    };

    let view_create_info = vk::ImageViewCreateInfo::default()
      .image(vk_image)
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(vk::Format::R8G8B8A8_SRGB)
      .subresource_range(vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        level_count: 1,
        layer_count: 1,
        ..Default::default()
      });
    let image_view = unsafe { device.create_image_view(&view_create_info, None).unwrap() };

    let sample_info = vk::SamplerCreateInfo::default()
      .mag_filter(vk::Filter::LINEAR)
      .min_filter(vk::Filter::LINEAR);
    let sampler = unsafe { device.create_sampler(&sample_info, None).unwrap() };

    let data = image.clone().into_raw();
    let mut buffer = Buffer::new(
      allocator,
      device,
      data.len() as u64,
      vk::BufferUsageFlags::TRANSFER_SRC,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )
    .unwrap();
    buffer.fill(&data).unwrap();

    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::default()
      .command_pool(pools.command_pool_graphics)
      .command_buffer_count(1);
    let copycmdbuffer =
      unsafe { device.allocate_command_buffers(&commandbuf_allocate_info) }.unwrap()[0];

    let cmdbegininfo =
      vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(copycmdbuffer, &cmdbegininfo) }?;

    //Insert commands here.
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
      })
      .buffer_offset(0)
      .buffer_row_length(0)
      .buffer_image_height(0)
      .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 });

    let barrier = vk::ImageMemoryBarrier::default()
      .src_access_mask(vk::AccessFlags::empty())
      .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
      .old_layout(vk::ImageLayout::UNDEFINED)
      .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
      .image(vk_image)
      .subresource_range(vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      });

    unsafe {
      device.cmd_pipeline_barrier(
        copycmdbuffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
      )
    }

    unsafe {
      device.cmd_copy_buffer_to_image(
        copycmdbuffer,
        buffer.buffer,
        vk_image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
      )
    };

    let barrier = vk::ImageMemoryBarrier::default()
      .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
      .dst_access_mask(vk::AccessFlags::SHADER_READ)
      .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
      .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
      .image(vk_image)
      .subresource_range(vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      });

    unsafe {
      device.cmd_pipeline_barrier(
        copycmdbuffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
      )
    }

    unsafe { device.end_command_buffer(copycmdbuffer) }?;
    let copycmdbuffers = [copycmdbuffer];
    let submit_infos = [vk::SubmitInfo::default()
      .command_buffers(&copycmdbuffers)];
    let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }?;
    unsafe { device.queue_submit(queues.graphics, &submit_infos, fence) }?;
    unsafe { device.wait_for_fences(&[fence], true, std::u64::MAX) }?;
    unsafe { device.destroy_fence(fence, None) };

    unsafe {
      device.destroy_buffer(buffer.buffer, None);
    }
    allocator.free(allocation).unwrap();

    unsafe { device.free_command_buffers(pools.command_pool_graphics, &[copycmdbuffer]) };

    Ok(Self {
      image,
      vk_image,
      image_view,
      sampler,
    })
  }

  pub fn destroy(&self, device: &ash::Device) {
    unsafe {
      device.destroy_sampler(self.sampler, None);
      device.destroy_image_view(self.image_view, None);
      device.destroy_image(self.vk_image, None);
    }
  }
}
