use std::mem::ManuallyDrop;

use ash::vk;
use glam as g;
use gpu_allocator::vulkan;

use crate::{
  buffer::Buffer,
  debug::DebugDong,
  model::{Model, TexturedInstanceData, TexturedVertexData},
  pipeline::{init_render_pass, Pipeline},
  pools::{create_command_buffers, Pools},
  queues::{init_instance, init_physical_device_and_properties, QueueFamilies, Queues},
  surface::SurfaceDong,
  swapchain::SwapchainDong,
  texture::Texture,
};

pub struct Aetna {
  pub window: winit::window::Window,
  #[allow(dead_code)]
  pub entry: ash::Entry,
  pub instance: ash::Instance,
  pub debug: ManuallyDrop<DebugDong>,
  pub surfaces: ManuallyDrop<SurfaceDong>,
  pub physical_device: vk::PhysicalDevice,
  //pub physical_device_properties: vk::PhysicalDeviceProperties,
  pub queue_families: QueueFamilies,
  pub queues: Queues,
  pub device: ash::Device,
  pub swapchain: SwapchainDong,
  pub render_pass: vk::RenderPass,
  pub pipeline: Pipeline,
  pub pools: Pools,
  pub command_buffers: Vec<vk::CommandBuffer>,
  pub allocator: ManuallyDrop<vulkan::Allocator>,
  pub models: Vec<Model<TexturedVertexData, TexturedInstanceData>>,
  pub uniform_buffer: Buffer,
  pub descriptor_pool: vk::DescriptorPool,
  pub descriptor_sets: Vec<vk::DescriptorSet>,
  pub descriptor_sets_texture: Vec<vk::DescriptorSet>,
  pub texture: Vec<Texture>,
}

impl Aetna {
  pub fn init(window: winit::window::Window) -> Result<Self, Box<dyn std::error::Error>> {
    let entry = unsafe { ash::Entry::load() }?;

    let layer_names: Vec<std::ffi::CString> =
      vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation")?];
    let mut debug_create_info = DebugDong::info();

    let instance = init_instance(&entry, &layer_names, &mut debug_create_info)?;
    let debug_messenger = DebugDong::init(&entry, &instance, &debug_create_info)?;
    let surface_dong = SurfaceDong::init(&entry, &instance, &window)?;
    let (physical_device, _properties) = init_physical_device_and_properties(&instance)?;
    let queue_families = QueueFamilies::init(&instance, physical_device, &surface_dong)?;
    let (logical_device, queues) = Queues::init(&instance, physical_device, &queue_families)?;
    let allocator_create_desc = gpu_allocator::vulkan::AllocatorCreateDesc {
      instance: instance.clone(),
      device: logical_device.clone(),
      physical_device,
      debug_settings: Default::default(),
      buffer_device_address: false,
      allocation_sizes: Default::default(),
    };
    let mut allocator = gpu_allocator::vulkan::Allocator::new(&allocator_create_desc)?;
    let mut swapchain_dong = SwapchainDong::init(
      &instance,
      physical_device,
      &logical_device,
      &surface_dong,
      &queue_families,
      &mut allocator,
    )?;
    let format = surface_dong
      .get_formats(physical_device)
      .unwrap()
      .first()
      .unwrap()
      .format;
    let render_pass = init_render_pass(&logical_device, format)?;
    swapchain_dong.create_frame_buffers(&logical_device, render_pass)?;
    let pipeline = Pipeline::init(&logical_device, &swapchain_dong, render_pass)?;
    let pools = Pools::init(&logical_device, &queue_families)?;

    let command_buffers =
      create_command_buffers(&logical_device, &pools, swapchain_dong.frame_buffers.len())?;

    let mut uniform_buffer = Buffer::new(
      &mut allocator,
      &logical_device,
      128,
      vk::BufferUsageFlags::UNIFORM_BUFFER,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;
    let camera_transform = [
      g::Mat4::IDENTITY.to_cols_array_2d(),
      g::Mat4::IDENTITY.to_cols_array_2d(),
    ];
    uniform_buffer.fill(&camera_transform)?;

    let pool_sizes = [
      vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(swapchain_dong.amount_of_images),
      vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(swapchain_dong.amount_of_images),
    ];
    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::default()
      .max_sets(2 * swapchain_dong.amount_of_images)
      .pool_sizes(&pool_sizes);
    let descriptor_pool =
      unsafe { logical_device.create_descriptor_pool(&descriptor_pool_create_info, None) }?;

    let desc_layouts =
      vec![pipeline.descriptor_set_layouts[0]; swapchain_dong.amount_of_images as usize];
    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
      .descriptor_pool(descriptor_pool)
      .set_layouts(&desc_layouts);
    let descriptor_sets =
      unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }?;

    for &descriptor_set in descriptor_sets.iter() {
      let buffer_info = [vk::DescriptorBufferInfo::default()
        .buffer(uniform_buffer.buffer)
        .offset(0)
        .range(128)];
      let write_descriptor_set = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(&buffer_info);
      unsafe {
        logical_device.update_descriptor_sets(&[write_descriptor_set], &[]);
      }
    }

    let desc_layouts_texture =
      vec![pipeline.descriptor_set_layouts[1]; swapchain_dong.amount_of_images as usize];
    let descriptor_set_allocate_info_texture = vk::DescriptorSetAllocateInfo::default()
      .descriptor_pool(descriptor_pool)
      .set_layouts(&desc_layouts_texture);
    let descriptor_sets_texture =
      unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_texture) }?;

    let texture = Texture::from_file(
      "assets/image.png",
      &mut allocator,
      &logical_device,
      &queues,
      &pools,
    )
    .unwrap();

    let texture1 = Texture::from_file(
      "assets/image1.png",
      &mut allocator,
      &logical_device,
      &queues,
      &pools,
    )
    .unwrap();

    for &descriptor_set in descriptor_sets_texture.iter() {
      let image_info = [
        vk::DescriptorImageInfo::default()
          .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
          .image_view(texture.image_view)
          .sampler(texture.sampler),
        vk::DescriptorImageInfo::default()
          .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
          .image_view(texture.image_view)
          .sampler(texture.sampler),
      ];
      let write_descriptor_set_texture = vk::WriteDescriptorSet::default()
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .dst_set(descriptor_set)
        .image_info(&image_info);
      unsafe {
        logical_device.update_descriptor_sets(&[write_descriptor_set_texture], &[]);
      }
    }

    Ok(Self {
      window,
      entry,
      instance,
      debug: std::mem::ManuallyDrop::new(debug_messenger),
      surfaces: std::mem::ManuallyDrop::new(surface_dong),
      physical_device,
      //physical_device_properties: properties,
      queue_families,
      queues,
      device: logical_device,
      swapchain: swapchain_dong,
      render_pass,
      pipeline,
      pools,
      command_buffers,
      allocator: std::mem::ManuallyDrop::new(allocator),
      models: vec![],
      uniform_buffer,
      descriptor_pool,
      descriptor_sets,
      descriptor_sets_texture,
      texture: vec![texture, texture1],
    })
  }

  pub fn update_command_buffer(&mut self, index: usize) -> Result<(), vk::Result> {
    self
      .swapchain
      .create_frame_buffers(&self.device, self.render_pass)?;

    let command_buffer = self.command_buffers[index];
    let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();
    unsafe {
      self
        .device
        .begin_command_buffer(command_buffer, &command_buffer_begin_info)
    }?;

    let clear_values = [
      vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 1.0],
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
      .render_pass(self.render_pass)
      .framebuffer(self.swapchain.frame_buffers[index])
      .render_area(vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: self.swapchain.extent,
      })
      .clear_values(&clear_values);

    unsafe {
      self.device.cmd_begin_render_pass(
        command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
      self.device.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.pipeline.pipeline,
      );

      self.device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.pipeline.pipeline_layout,
        0,
        &[
          self.descriptor_sets[index],
          self.descriptor_sets_texture[index],
        ],
        &[],
      );

      for model in &self.models {
        model.draw(&self.device, command_buffer);
      }

      self.device.cmd_end_render_pass(command_buffer);
      self.device.end_command_buffer(command_buffer)?;
    }

    Ok(())
  }

  pub fn recreate_swapchain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
      self.device.device_wait_idle()?;
    }
    unsafe {
      self.swapchain.cleanup(&self.device, &mut self.allocator);
    }
    self.swapchain = SwapchainDong::init(
      &self.instance,
      self.physical_device,
      &self.device,
      &self.surfaces,
      &self.queue_families,
      &mut self.allocator,
    )?;

    unsafe {
      self.pipeline.cleanup(&self.device);
    }
    self.pipeline = Pipeline::init(&self.device, &self.swapchain, self.render_pass)?;

    Ok(())
  }
}

impl Drop for Aetna {
  fn drop(&mut self) {
    unsafe {
      self
        .device
        .device_wait_idle()
        .expect("Unable to wait for device idle");
      for texture in &self.texture {
        texture.destroy(&self.device);
      }
      self
        .device
        .destroy_descriptor_pool(self.descriptor_pool, None);
      self.device.destroy_buffer(self.uniform_buffer.buffer, None);
      self
        .allocator
        .free(std::mem::take(&mut self.uniform_buffer.allocation))
        .unwrap();
      for model in &mut self.models {
        model.cleanup(&self.device, &mut self.allocator);
      }
      self
        .device
        .free_command_buffers(self.pools.command_pool_graphics, &self.command_buffers);
      self.pools.cleanup(&self.device);
      self.pipeline.cleanup(&self.device);
      self.device.destroy_render_pass(self.render_pass, None);
      self.swapchain.cleanup(&self.device, &mut self.allocator);
      std::mem::ManuallyDrop::drop(&mut self.allocator);
      self.device.destroy_device(None);
      std::mem::ManuallyDrop::drop(&mut self.surfaces);
      std::mem::ManuallyDrop::drop(&mut self.debug);
      self.instance.destroy_instance(None);
    }
  }
}
