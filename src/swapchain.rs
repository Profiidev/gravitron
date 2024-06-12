
use ash::{vk, khr};
use gpu_allocator::vulkan;

use crate::surface::SurfaceDong;
use crate::queues::QueueFamilies;

pub struct SwapchainDong {
  pub loader: khr::swapchain::Device,
  pub swapchain: vk::SwapchainKHR,
  pub images: Vec<vk::Image>,
  pub image_views: Vec<vk::ImageView>,
  pub depth_image: vk::Image,
  pub depth_image_allocation: vulkan::Allocation,
  pub depth_image_view: vk::ImageView,
  pub frame_buffers: Vec<vk::Framebuffer>,
  //pub surface_format: vk::SurfaceFormatKHR,
  pub extent: vk::Extent2D,
  pub image_available: Vec<vk::Semaphore>,
  pub render_finished: Vec<vk::Semaphore>,
  pub may_begin_drawing: Vec<vk::Fence>,
  pub amount_of_images: u32,
  pub current_image: usize,
}

impl SwapchainDong {
  pub fn init(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    surfaces: &SurfaceDong,
    queue_families: &QueueFamilies,
    allocator: &mut vulkan::Allocator,
  ) -> Result<Self, vk::Result> {
    let surface_capabilities = surfaces.get_capabilities(physical_device)?;
    //let surface_present_modes = surfaces.get_present_modes(physical_device)?;
    let surface_format = *surfaces.get_formats(physical_device)?.first().unwrap();

    let mut extent = surface_capabilities.current_extent;
    if extent.width == u32::MAX || extent.height == u32::MAX {
      extent.width = 800;
      extent.height = 600;
    }

    let queue_families = [queue_families.graphics_q_index.unwrap()];
    let image_count =
      if surface_capabilities.min_image_count <= surface_capabilities.max_image_count {
        3.max(surface_capabilities.min_image_count)
          .min(surface_capabilities.max_image_count)
      } else {
        surface_capabilities.min_image_count
      };
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
      .surface(surfaces.surface)
      .min_image_count(image_count)
      .image_format(surface_format.format)
      .image_color_space(surface_format.color_space)
      .image_extent(extent)
      .image_array_layers(1)
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
      .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
      .queue_family_indices(&queue_families)
      .pre_transform(surface_capabilities.current_transform)
      .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
      .present_mode(vk::PresentModeKHR::MAILBOX);

    let swapchain_loader = khr::swapchain::Device::new(instance, logical_device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;

    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
    let amount_of_images = swapchain_images.len() as u32;

    let mut swapchain_image_views = Vec::with_capacity(swapchain_images.len());
    for image in &swapchain_images {
      let subresource_range = vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);
      let image_view_create_info = vk::ImageViewCreateInfo::default()
        .image(*image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(surface_format.format)
        .subresource_range(subresource_range);
      let image_view = unsafe { logical_device.create_image_view(&image_view_create_info, None) }?;
      swapchain_image_views.push(image_view);
    }

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

    let depth_image = unsafe { logical_device.create_image(&depth_image_create_info, None) }?;
    let requirements = unsafe { logical_device.get_image_memory_requirements(depth_image) };
    let allocation_create_desc = vulkan::AllocationCreateDesc {
      requirements,
      location: gpu_allocator::MemoryLocation::GpuOnly,
      linear: true,
      allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
      name: "Depth Image",
    };
    let depth_image_allocation = allocator.allocate(&allocation_create_desc).unwrap();
    unsafe {
      logical_device.bind_image_memory(
        depth_image,
        depth_image_allocation.memory(),
        depth_image_allocation.offset(),
      )
    }?;

    let subresource_range = vk::ImageSubresourceRange::default()
      .aspect_mask(vk::ImageAspectFlags::DEPTH)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1);
    let depth_image_view_create_info = vk::ImageViewCreateInfo::default()
      .image(depth_image)
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(vk::Format::D32_SFLOAT)
      .subresource_range(subresource_range);
    let depth_image_view =
      unsafe { logical_device.create_image_view(&depth_image_view_create_info, None) }?;

    let mut image_available = Vec::new();
    let mut render_finished = Vec::new();
    let mut may_begin_drawing = Vec::new();
    let semaphore_create_info = vk::SemaphoreCreateInfo::default();
    let fence_create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    for _ in 0..amount_of_images {
      let image_available_semaphore =
        unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }?;
      let render_finished_semaphore =
        unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }?;
      image_available.push(image_available_semaphore);
      render_finished.push(render_finished_semaphore);
      let fence = unsafe { logical_device.create_fence(&fence_create_info, None) }?;
      may_begin_drawing.push(fence);
    }

    Ok(Self {
      loader: swapchain_loader,
      swapchain,
      images: swapchain_images,
      image_views: swapchain_image_views,
      frame_buffers: Vec::new(),
      //surface_format,
      extent,
      image_available,
      render_finished,
      amount_of_images,
      current_image: 0,
      may_begin_drawing,
      depth_image,
      depth_image_allocation,
      depth_image_view,
    })
  }

  pub unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &mut vulkan::Allocator) {
    logical_device.destroy_image_view(self.depth_image_view, None);
    logical_device.destroy_image(self.depth_image, None);
    allocator
      .free(std::mem::take(&mut self.depth_image_allocation))
      .unwrap();

    for fence in &self.may_begin_drawing {
      logical_device.destroy_fence(*fence, None);
    }
    for semaphore in &self.image_available {
      logical_device.destroy_semaphore(*semaphore, None);
    }
    for semaphore in &self.render_finished {
      logical_device.destroy_semaphore(*semaphore, None);
    }
    for frame_buffer in &self.frame_buffers {
      logical_device.destroy_framebuffer(*frame_buffer, None);
    }
    for image_view in &self.image_views {
      logical_device.destroy_image_view(*image_view, None);
    }
    self.loader.destroy_swapchain(self.swapchain, None);
  }

  pub fn create_frame_buffers(
    &mut self,
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
  ) -> Result<(), vk::Result> {
    for image_view in &self.image_views {
      let vi = [*image_view, self.depth_image_view];
      let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
        .render_pass(render_pass)
        .attachments(&vi)
        .width(self.extent.width)
        .height(self.extent.height)
        .layers(1);
      let frame_buffer =
        unsafe { logical_device.create_framebuffer(&frame_buffer_create_info, None) }?;
      self.frame_buffers.push(frame_buffer);
    }
    Ok(())
  }
}