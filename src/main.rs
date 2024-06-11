use std::mem::ManuallyDrop;

use ash::{ext, khr, vk};
use gpu_allocator::vulkan;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
  raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  event_loop
    .run_app(&mut App {
      aetna: None,
      frame: 0,
      start_time: std::time::Instant::now(),
    })
    .unwrap();

  Ok(())
}

struct App {
  aetna: Option<Aetna>,
  frame: u64,
  start_time: std::time::Instant,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title("Vulkan")
      .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));

    let window = event_loop.create_window(window_attributes).unwrap();
    self.aetna = Some(Aetna::init(window).unwrap());
  }

  fn window_event(
    &mut self,
    _event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        std::mem::drop(self.aetna.take());
      }
      winit::event::WindowEvent::RedrawRequested => {
        if let Some(aetna) = self.aetna.as_mut() {
          unsafe {
            aetna
              .device
              .wait_for_fences(
                &[aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image]],
                true,
                std::u64::MAX,
              )
              .expect("Unable to wait for fences");

            aetna
              .device
              .reset_fences(&[aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image]])
              .expect("Unable to reset fences");
          }

          let (image_index, _) = unsafe {
            aetna
              .swapchain
              .loader
              .acquire_next_image(
                aetna.swapchain.swapchain,
                std::u64::MAX,
                aetna.swapchain.image_available[aetna.swapchain.current_image],
                vk::Fence::null(),
              )
              .expect("Unable to acquire next image")
          };

          let semaphore_available =
            [aetna.swapchain.image_available[aetna.swapchain.current_image]];
          let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
          let semaphore_render_finished =
            [aetna.swapchain.render_finished[aetna.swapchain.current_image]];
          let command_buffer = [aetna.command_buffers[aetna.swapchain.current_image]];

          let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&semaphore_available)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffer)
            .signal_semaphores(&semaphore_render_finished)];

          unsafe {
            aetna
              .device
              .queue_submit(
                aetna.queues.graphics,
                &submit_info,
                aetna.swapchain.may_begin_drawing[aetna.swapchain.current_image],
              )
              .expect("Unable to submit queue");
          }

          let swapchains = [aetna.swapchain.swapchain];
          let image_indices = [image_index];
          let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&semaphore_render_finished)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
          unsafe {
            aetna
              .swapchain
              .loader
              .queue_present(aetna.queues.graphics, &present_info)
              .expect("Unable to queue present");
          }

          aetna.swapchain.current_image =
            (aetna.swapchain.current_image + 1) % aetna.swapchain.amount_of_images as usize;

          self.frame += 1;
          let elapsed = self.start_time.elapsed();
          if elapsed.as_secs() >= 1 {
            println!("FPS: {}", self.frame);
            self.frame = 0;
            self.start_time = std::time::Instant::now();
          }
          
          let max_frames = 165;
          let frame_time = std::time::Duration::from_secs(1) / max_frames;
          let elapsed = self.start_time.elapsed();
          if elapsed < frame_time * self.frame as u32 {
            std::thread::sleep(frame_time * self.frame as u32 - elapsed);
          }
        }
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
    if let Some(aetna) = self.aetna.as_mut() {
      aetna.window.request_redraw();
    }
  }
}

fn init_instance(
  entry: &ash::Entry,
  layer_names: &[std::ffi::CString],
  debug_create_info: &mut vk::DebugUtilsMessengerCreateInfoEXT,
) -> Result<ash::Instance, Box<dyn std::error::Error>> {
  let engine_name = std::ffi::CString::new("Vulkan Engine")?;
  let app_name = std::ffi::CString::new("Test App")?;

  let app_info = vk::ApplicationInfo::default()
    .application_name(&app_name)
    .engine_name(&engine_name)
    .engine_version(vk::make_api_version(0, 0, 42, 0))
    .application_version(vk::make_api_version(0, 0, 1, 0))
    .api_version(vk::make_api_version(0, 1, 3, 278));

  let layer_name_ptrs: Vec<*const i8> = layer_names
    .iter()
    .map(|layer_name| layer_name.as_ptr())
    .collect();
  let extension_name_ptrs = [
    ext::debug_utils::NAME.as_ptr(),
    khr::wayland_surface::NAME.as_ptr(),
    khr::surface::NAME.as_ptr(),
  ];

  let instance_create_info = vk::InstanceCreateInfo::default()
    .push_next(debug_create_info)
    .application_info(&app_info)
    .enabled_layer_names(&layer_name_ptrs)
    .enabled_extension_names(&extension_name_ptrs);

  Ok(unsafe { entry.create_instance(&instance_create_info, None) }?)
}

fn init_physical_device_and_properties(
  instance: &ash::Instance,
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties), vk::Result> {
  let phys_devices = unsafe { instance.enumerate_physical_devices() }?;

  let mut physical_device = None;
  for p in phys_devices {
    let properties = unsafe { instance.get_physical_device_properties(p) };
    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
      physical_device = Some((p, properties));
      break;
    }
  }
  Ok(physical_device.unwrap())
}

struct DebugDong {
  loader: ext::debug_utils::Instance,
  messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugDong {
  fn info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
      .message_severity(
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
      )
      .message_type(
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
          | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
          | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
      )
      .pfn_user_callback(Some(vulkan_debug_utils_callback))
  }

  fn init(
    entry: &ash::Entry,
    instance: &ash::Instance,
    debug_create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let loader = ext::debug_utils::Instance::new(entry, instance);
    let messenger = unsafe { loader.create_debug_utils_messenger(debug_create_info, None) }?;
    Ok(Self { loader, messenger })
  }
}

impl Drop for DebugDong {
  fn drop(&mut self) {
    unsafe {
      self
        .loader
        .destroy_debug_utils_messenger(self.messenger, None);
    }
  }
}

struct SurfaceDong {
  surface_loader: khr::surface::Instance,
  surface: vk::SurfaceKHR,
}

impl SurfaceDong {
  fn init(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let display_handle = window.display_handle().unwrap().as_raw();
    let window_handle = window.window_handle().unwrap().as_raw();
    let surface =
      unsafe { ash_window::create_surface(entry, instance, display_handle, window_handle, None) }?;
    let surface_loader = khr::surface::Instance::new(entry, instance);

    Ok(Self {
      surface_loader,
      surface,
    })
  }

  fn get_capabilities(
    &self,
    physical_device: vk::PhysicalDevice,
  ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
    unsafe {
      self
        .surface_loader
        .get_physical_device_surface_capabilities(physical_device, self.surface)
    }
  }
/*
  fn get_present_modes(
    &self,
    physical_device: vk::PhysicalDevice,
  ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
    unsafe {
      self
        .surface_loader
        .get_physical_device_surface_present_modes(physical_device, self.surface)
    }
  }
*/
  fn get_formats(
    &self,
    physical_device: vk::PhysicalDevice,
  ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
    unsafe {
      self
        .surface_loader
        .get_physical_device_surface_formats(physical_device, self.surface)
    }
  }

  fn get_support(
    &self,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
  ) -> Result<bool, vk::Result> {
    unsafe {
      self.surface_loader.get_physical_device_surface_support(
        physical_device,
        queue_family_index,
        self.surface,
      )
    }
  }
}

impl Drop for SurfaceDong {
  fn drop(&mut self) {
    unsafe {
      self.surface_loader.destroy_surface(self.surface, None);
    }
  }
}

struct QueueFamilies {
  graphics_q_index: Option<u32>,
  transfer_q_index: Option<u32>,
}

impl QueueFamilies {
  fn init(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_dong: &SurfaceDong,
  ) -> Result<Self, vk::Result> {
    let queue_family_properties =
      unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut queue_family_index_graphics = None;
    let mut queue_family_index_transfer = None;
    for (i, properties) in queue_family_properties.iter().enumerate() {
      if properties.queue_count > 0
        && properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        && surface_dong.get_support(physical_device, i as u32)?
      {
        queue_family_index_graphics = Some(i as u32);
      }
      if properties.queue_count > 0
        && properties.queue_flags.contains(vk::QueueFlags::TRANSFER)
        && (queue_family_index_transfer.is_none()
          || !properties.queue_flags.contains(vk::QueueFlags::GRAPHICS))
      {
        queue_family_index_transfer = Some(i as u32);
      }
    }

    Ok(Self {
      graphics_q_index: queue_family_index_graphics,
      transfer_q_index: queue_family_index_transfer,
    })
  }
}

struct Queues {
  graphics: vk::Queue,
  #[allow(dead_code)]
  transfer: vk::Queue,
}

impl Queues {
  fn init(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
  ) -> Result<(ash::Device, Self), vk::Result> {
    let queue_priorities = [1.0];
    let queue_create_infos = [
      vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_families.graphics_q_index.unwrap())
        .queue_priorities(&queue_priorities),
      vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_families.transfer_q_index.unwrap())
        .queue_priorities(&queue_priorities),
    ];
    let device_extension_name_ptrs = [khr::swapchain::NAME.as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::default()
      .queue_create_infos(&queue_create_infos)
      .enabled_extension_names(&device_extension_name_ptrs);

    let logical_device =
      unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
    let graphics_queue =
      unsafe { logical_device.get_device_queue(queue_families.graphics_q_index.unwrap(), 0) };
    let transfer_queue =
      unsafe { logical_device.get_device_queue(queue_families.transfer_q_index.unwrap(), 0) };

    Ok((
      logical_device,
      Self {
        graphics: graphics_queue,
        transfer: transfer_queue,
      },
    ))
  }
}

struct SwapchainDong {
  loader: khr::swapchain::Device,
  swapchain: vk::SwapchainKHR,
  //images: Vec<vk::Image>,
  image_views: Vec<vk::ImageView>,
  frame_buffers: Vec<vk::Framebuffer>,
  //surface_format: vk::SurfaceFormatKHR,
  extent: vk::Extent2D,
  image_available: Vec<vk::Semaphore>,
  render_finished: Vec<vk::Semaphore>,
  may_begin_drawing: Vec<vk::Fence>,
  amount_of_images: u32,
  current_image: usize,
}

impl SwapchainDong {
  fn init(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    surfaces: &SurfaceDong,
    queue_families: &QueueFamilies,
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
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
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
      //images: swapchain_images,
      image_views: swapchain_image_views,
      frame_buffers: Vec::new(),
      //surface_format,
      extent,
      image_available,
      render_finished,
      amount_of_images,
      current_image: 0,
      may_begin_drawing,
    })
  }

  unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
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

  fn create_frame_buffers(
    &mut self,
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
  ) -> Result<(), vk::Result> {
    for image_view in &self.image_views {
      let vi = [*image_view];
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

fn init_render_pass(
  surface_dong: &SurfaceDong,
  physical_device: vk::PhysicalDevice,
  logical_device: &ash::Device,
) -> Result<vk::RenderPass, vk::Result> {
  let attachment = [vk::AttachmentDescription::default()
    .format(
      surface_dong
        .get_formats(physical_device)?
        .first()
        .unwrap()
        .format,
    )
    .samples(vk::SampleCountFlags::TYPE_1)
    .load_op(vk::AttachmentLoadOp::CLEAR)
    .store_op(vk::AttachmentStoreOp::STORE)
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
    .initial_layout(vk::ImageLayout::UNDEFINED)
    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

  let color_attachment_ref = [vk::AttachmentReference::default()
    .attachment(0)
    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

  let subpass = [vk::SubpassDescription::default()
    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    .color_attachments(&color_attachment_ref)];

  let subpass_dependency = [vk::SubpassDependency::default()
    .src_subpass(vk::SUBPASS_EXTERNAL)
    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    .dst_subpass(0)
    .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    .dst_access_mask(
      vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
    )];

  let render_pass_create_info = vk::RenderPassCreateInfo::default()
    .attachments(&attachment)
    .subpasses(&subpass)
    .dependencies(&subpass_dependency);
  unsafe { logical_device.create_render_pass(&render_pass_create_info, None) }
}

struct Pipeline {
  pipeline: vk::Pipeline,
  pipeline_layout: vk::PipelineLayout,
}

impl Pipeline {
  fn init(
    logical_device: &ash::Device,
    swapchain_dong: &SwapchainDong,
    render_pass: vk::RenderPass,
  ) -> Result<Self, vk::Result> {
    let vertex_shader_create_info = vk::ShaderModuleCreateInfo::default()
      .code(vk_shader_macros::include_glsl!("./shaders/shader.vert"));
    let vertex_shader_module =
      unsafe { logical_device.create_shader_module(&vertex_shader_create_info, None) }?;

    let fragment_shader_create_info = vk::ShaderModuleCreateInfo::default()
      .code(vk_shader_macros::include_glsl!("./shaders/shader.frag"));
    let fragment_shader_module =
      unsafe { logical_device.create_shader_module(&fragment_shader_create_info, None) }?;

    let main_function_name = std::ffi::CString::new("main").unwrap();
    let vertex_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
      .stage(vk::ShaderStageFlags::VERTEX)
      .module(vertex_shader_module)
      .name(&main_function_name);
    let fragment_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
      .stage(vk::ShaderStageFlags::FRAGMENT)
      .module(fragment_shader_module)
      .name(&main_function_name);
    let shader_stages = [
      vertex_shader_stage_create_info,
      fragment_shader_stage_create_info,
    ];

    let vertex_attrib_descs = [
      vk::VertexInputAttributeDescription::default()
        .binding(0)
        .location(0)
        .offset(0)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(1)
        .offset(0)
        .format(vk::Format::R32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(2)
        .offset(4)
        .format(vk::Format::R32G32B32A32_SFLOAT),
    ];

    let vertex_binding_descs = [
      vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(4 * 4)
        .input_rate(vk::VertexInputRate::VERTEX),
      vk::VertexInputBindingDescription::default()
        .binding(1)
        .stride(4 * 5)
        .input_rate(vk::VertexInputRate::VERTEX),
    ];

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
      .vertex_binding_descriptions(&vertex_binding_descs)
      .vertex_attribute_descriptions(&vertex_attrib_descs);
    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
      .topology(vk::PrimitiveTopology::POINT_LIST);

    let viewport = [vk::Viewport::default()
      .x(0.0)
      .y(0.0)
      .width(swapchain_dong.extent.width as f32)
      .height(swapchain_dong.extent.height as f32)
      .min_depth(0.0)
      .max_depth(1.0)];
    let scissor = [vk::Rect2D::default()
      .offset(vk::Offset2D::default())
      .extent(swapchain_dong.extent)];

    let viewport_info = vk::PipelineViewportStateCreateInfo::default()
      .viewports(&viewport)
      .scissors(&scissor);

    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
      .line_width(1.0)
      .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
      .cull_mode(vk::CullModeFlags::NONE)
      .polygon_mode(vk::PolygonMode::FILL);

    let multisample_info = vk::PipelineMultisampleStateCreateInfo::default()
      .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let color_blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
      .color_write_mask(
        vk::ColorComponentFlags::R
          | vk::ColorComponentFlags::G
          | vk::ColorComponentFlags::B
          | vk::ColorComponentFlags::A,
      )
      .blend_enable(false)
      .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .color_blend_op(vk::BlendOp::ADD)
      .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .alpha_blend_op(vk::BlendOp::ADD)];
    let color_blend_info =
      vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachment);

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default();
    let pipeline_layout =
      unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
      .stages(&shader_stages)
      .vertex_input_state(&vertex_input_info)
      .input_assembly_state(&input_assembly_info)
      .viewport_state(&viewport_info)
      .rasterization_state(&rasterizer_info)
      .multisample_state(&multisample_info)
      .color_blend_state(&color_blend_info)
      .layout(pipeline_layout)
      .render_pass(render_pass)
      .subpass(0);

    let pipeline = unsafe {
      logical_device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        .expect("Unable to create graphics pipeline")
    }[0];

    unsafe {
      logical_device.destroy_shader_module(vertex_shader_module, None);
      logical_device.destroy_shader_module(fragment_shader_module, None);
    }

    Ok(Self {
      pipeline,
      pipeline_layout,
    })
  }

  unsafe fn cleanup(&self, logical_device: &ash::Device) {
    logical_device.destroy_pipeline(self.pipeline, None);
    logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
  }
}

struct Pools {
  command_pool_graphics: vk::CommandPool,
  command_pool_transfer: vk::CommandPool,
}

impl Pools {
  fn init(
    logical_device: &ash::Device,
    queue_families: &QueueFamilies,
  ) -> Result<Self, vk::Result> {
    let command_pool_create_info = vk::CommandPoolCreateInfo::default()
      .queue_family_index(queue_families.graphics_q_index.unwrap())
      .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let command_pool_graphics =
      unsafe { logical_device.create_command_pool(&command_pool_create_info, None) }?;

    let command_pool_create_info = vk::CommandPoolCreateInfo::default()
      .queue_family_index(queue_families.transfer_q_index.unwrap())
      .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let command_pool_transfer =
      unsafe { logical_device.create_command_pool(&command_pool_create_info, None) }?;

    Ok(Self {
      command_pool_graphics,
      command_pool_transfer,
    })
  }

  unsafe fn cleanup(&self, logical_device: &ash::Device) {
    logical_device.destroy_command_pool(self.command_pool_graphics, None);
    logical_device.destroy_command_pool(self.command_pool_transfer, None);
  }
}

fn create_command_buffers(
  logical_device: &ash::Device,
  pools: &Pools,
  amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
  let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
    .command_pool(pools.command_pool_graphics)
    .command_buffer_count(amount as u32);
  unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
}

fn fill_command_buffers(
  logical_device: &ash::Device,
  command_buffers: &[vk::CommandBuffer],
  swapchain_dong: &SwapchainDong,
  render_pass: vk::RenderPass,
  pipeline: &Pipeline,
  vb: &vk::Buffer,
  vb1: &vk::Buffer,
) -> Result<(), vk::Result> {
  for (i, &command_buffer) in command_buffers.iter().enumerate() {
    let begin_info = vk::CommandBufferBeginInfo::default();
    unsafe { logical_device.begin_command_buffer(command_buffer, &begin_info) }?;

    let clear_values = [vk::ClearValue {
      color: vk::ClearColorValue {
        float32: [0.0, 0.0, 0.0, 1.0],
      },
    }];
    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
      .render_pass(render_pass)
      .framebuffer(swapchain_dong.frame_buffers[i])
      .render_area(vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_dong.extent,
      })
      .clear_values(&clear_values);
    unsafe {
      logical_device.cmd_begin_render_pass(
        command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
      logical_device.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        pipeline.pipeline,
      );
      logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &[*vb], &[0]);
      logical_device.cmd_bind_vertex_buffers(command_buffer, 1, &[*vb1], &[0]);
      logical_device.cmd_draw(command_buffer, 3, 1, 0, 0);
      logical_device.cmd_end_render_pass(command_buffer);
      logical_device.end_command_buffer(command_buffer)?;
    }
  }
  Ok(())
}

struct Buffer {
  buffer: vk::Buffer,
  allocation: vulkan::Allocation,
}

impl Buffer {
  fn new(
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    size: u64,
    usage: vk::BufferUsageFlags,
    memory_location: gpu_allocator::MemoryLocation,
  ) -> Result<Self, vk::Result> {
    let buffer_create_info = vk::BufferCreateInfo::default().size(size).usage(usage);
    let buffer = unsafe { device.create_buffer(&buffer_create_info, None) }?;
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation_create_desc = vulkan::AllocationCreateDesc {
      requirements,
      location: memory_location,
      linear: true,
      allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
      name: "Buffer",
    };
    let allocation = allocator.allocate(&allocation_create_desc).unwrap();

    unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }?;

    Ok(Self { buffer, allocation })
  }

  fn fill<T: Sized>(
    &mut self,
    data: &[T],
  ) -> Result<(), vk::Result> {
    let data_ptr = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
    unsafe {
      data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
    }
    Ok(())
  }
}

struct Aetna {
  window: winit::window::Window,
  #[allow(dead_code)]
  entry: ash::Entry,
  instance: ash::Instance,
  debug: ManuallyDrop<DebugDong>,
  surfaces: ManuallyDrop<SurfaceDong>,
  //physical_device: vk::PhysicalDevice,
  //physical_device_properties: vk::PhysicalDeviceProperties,
  //queue_families: QueueFamilies,
  queues: Queues,
  device: ash::Device,
  swapchain: SwapchainDong,
  render_pass: vk::RenderPass,
  pipeline: Pipeline,
  pools: Pools,
  command_buffers: Vec<vk::CommandBuffer>,
  allocator: ManuallyDrop<vulkan::Allocator>,
  buffers: Vec<Buffer>,
}

impl Aetna {
  fn init(window: winit::window::Window) -> Result<Self, Box<dyn std::error::Error>> {
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
    let mut swapchain_dong = SwapchainDong::init(
      &instance,
      physical_device,
      &logical_device,
      &surface_dong,
      &queue_families,
    )?;
    let render_pass = init_render_pass(&surface_dong, physical_device, &logical_device)?;
    swapchain_dong.create_frame_buffers(&logical_device, render_pass)?;
    let pipeline = Pipeline::init(&logical_device, &swapchain_dong, render_pass)?;
    let pools = Pools::init(&logical_device, &queue_families)?;

    let allocator_create_desc = gpu_allocator::vulkan::AllocatorCreateDesc {
      instance: instance.clone(),
      device: logical_device.clone(),
      physical_device,
      debug_settings: Default::default(),
      buffer_device_address: false,
      allocation_sizes: Default::default(),
    };
    let mut allocator = gpu_allocator::vulkan::Allocator::new(&allocator_create_desc)?;

    let data = [
      0.4f32, -0.2f32, 0.0f32, 1.0f32, 0.8f32, 0.0f32, 0.0f32, 1.0f32, -0.4f32, 0.2f32, 0.0f32,
      1.0f32,
    ];
    let data1 = [5.0, 1.0, 0.0, 1.0, 1.0_f32];

    let mut buffer = Buffer::new(
      &mut allocator,
      &logical_device,
      std::mem::size_of_val(&data) as u64,
      vk::BufferUsageFlags::VERTEX_BUFFER,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;
    buffer.fill(&data)?;

    let mut buffer1 = Buffer::new(
      &mut allocator,
      &logical_device,
      std::mem::size_of_val(&data1) as u64,
      vk::BufferUsageFlags::VERTEX_BUFFER,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;
    buffer1.fill(&data1)?;

    let command_buffers =
      create_command_buffers(&logical_device, &pools, swapchain_dong.frame_buffers.len())?;
    fill_command_buffers(
      &logical_device,
      &command_buffers,
      &swapchain_dong,
      render_pass,
      &pipeline,
      &buffer.buffer,
      &buffer1.buffer,
    )?;

    Ok(Self {
      window,
      entry,
      instance,
      debug: std::mem::ManuallyDrop::new(debug_messenger),
      surfaces: std::mem::ManuallyDrop::new(surface_dong),
      //physical_device,
      //physical_device_properties: properties,
      //queue_families,
      queues,
      device: logical_device,
      swapchain: swapchain_dong,
      render_pass,
      pipeline,
      pools,
      command_buffers,
      allocator: std::mem::ManuallyDrop::new(allocator),
      buffers: vec![buffer, buffer1],
    })
  }
}

impl Drop for Aetna {
  fn drop(&mut self) {
    unsafe {
      self
        .device
        .device_wait_idle()
        .expect("Unable to wait for device idle");
      for buffer in &mut self.buffers {
        self
          .allocator
          .free(std::mem::take(&mut buffer.allocation)).unwrap();
        self.device.destroy_buffer(buffer.buffer, None);
      }
      std::mem::ManuallyDrop::drop(&mut self.allocator);
      self
        .device
        .free_command_buffers(self.pools.command_pool_graphics, &self.command_buffers);
      self.pools.cleanup(&self.device);
      self.pipeline.cleanup(&self.device);
      self.device.destroy_render_pass(self.render_pass, None);
      self.swapchain.cleanup(&self.device);
      self.device.destroy_device(None);
      std::mem::ManuallyDrop::drop(&mut self.surfaces);
      std::mem::ManuallyDrop::drop(&mut self.debug);
      self.instance.destroy_instance(None);
    }
  }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
  message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
  message_type: vk::DebugUtilsMessageTypeFlagsEXT,
  p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
  _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
  let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
  let severity = format!("{:?}", message_severity).to_lowercase();
  let ty = format!("{:?}", message_type).to_lowercase();
  println!("[Debug][{}][{}] {:?}", severity, ty, message);
  vk::FALSE
}
