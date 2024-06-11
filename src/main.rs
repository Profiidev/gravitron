use std::mem::ManuallyDrop;

use ash::{ext, khr, vk};
use glam as g;
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
      handle: 0,
      camera: Camera::builder().build(),
    })
    .unwrap();

  Ok(())
}

struct App {
  aetna: Option<Aetna>,
  frame: u64,
  start_time: std::time::Instant,
  handle: usize,
  camera: Camera,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title("Vulkan")
      .with_inner_size(Size::Logical(LogicalSize::new(800.0, 600.0)));

    let window = event_loop.create_window(window_attributes).unwrap();
    let mut aetna = Aetna::init(window).unwrap();

    let mut cube = Model::cube();
    
    let scale = g::Mat4::from_scale(g::Vec3::from_array([0.1, 0.1, 0.1]));
    let scale_2 = g::Mat4::from_scale(g::Vec3::from_array([0.02, 0.02, 0.02]));
    let scale_3 = g::Mat4::from_scale(g::Vec3::from_array([0.03, 0.03, 0.03]));
    for i in 0..10 {
      for j in 0..10 {
        self.handle = cube.insert_visibly(InstanceData {
          model_matrix: (g::Mat4::from_translation(g::Vec3::new(
            i as f32 * 0.2 - 1.0,
            j as f32 * 0.2 - 1.0,
            0.5,
          )) * scale_3)
            .to_cols_array_2d(),
          color: [1.0, i as f32 * 0.07, j as f32 * 0.07],
        });
        self.handle = cube.insert_visibly(InstanceData {
          model_matrix: (g::Mat4::from_translation(g::Vec3::new(
            i as f32 * 0.2 - 1.0,
            0.0,
            j as f32 * 0.2 - 1.0,
          )) * scale_2)
            .to_cols_array_2d(),
          color: [i as f32 * 0.07, j as f32 * 0.07, 1.0],
        });
      }
    }

    self.handle = cube.insert_visibly(InstanceData {
      model_matrix: (g::Mat4::from_translation(g::Vec3::new(0.5, 0.0, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.5, 0.01, 0.01])))
      .to_cols_array_2d(),
      color: [1.0, 0.5, 0.5],
    });
    self.handle = cube.insert_visibly(InstanceData {
      model_matrix: (g::Mat4::from_translation(g::Vec3::new(0.0, 0.5, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.01, 0.5, 0.01])))
      .to_cols_array_2d(),
      color: [0.5, 1.0, 0.5],
    });
    self.handle = cube.insert_visibly(InstanceData {
      model_matrix: (g::Mat4::from_translation(g::Vec3::new(0.0, 0.0, 0.0))
        * g::Mat4::from_scale(g::Vec3::from_array([0.01, 0.01, 0.5])))
      .to_cols_array_2d(),
      color: [0.5, 0.5, 1.0],
    });

    cube
      .update_vertex_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();
    cube
      .update_instance_buffer(&mut aetna.allocator, &aetna.device)
      .unwrap();

    let models = vec![cube];
    aetna.models = models;

    self.aetna = Some(aetna);
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
      winit::event::WindowEvent::KeyboardInput {
        device_id: _,
        event:
          winit::event::KeyEvent {
            logical_key: key,
            state: winit::event::ElementState::Pressed,
            ..
          },
        is_synthetic: _,
      } => match key.as_ref() {
        winit::keyboard::Key::Character("w") => {
          self.camera.move_forward(0.05);
        }
        winit::keyboard::Key::Character("s") => {
          self.camera.move_backward(0.05);
        }
        winit::keyboard::Key::Character("a") => {
          self.camera.move_left(0.05);
        }
        winit::keyboard::Key::Character("d") => {
          self.camera.move_right(0.05);
        }
        winit::keyboard::Key::Character("q") => {
          self.camera.move_up(0.05);
        }
        winit::keyboard::Key::Character("e") => {
          self.camera.move_down(0.05);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
          self.camera.turn_up(0.1);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
          self.camera.turn_down(0.1);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
          self.camera.turn_left(0.05);
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
          self.camera.turn_right(0.05);
        }
        _ => {}
      },
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

          self.camera.update_buffer(&mut aetna.uniform_buffer).unwrap();

          for m in &mut aetna.models {
            m.update_instance_buffer(&mut aetna.allocator, &aetna.device)
              .unwrap();
          }

          aetna
            .update_command_buffer(aetna.swapchain.current_image)
            .expect("Unable to update command buffer");

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
    khr::surface::NAME.as_ptr(),
    #[cfg(target_os = "linux")]
    khr::wayland_surface::NAME.as_ptr(),
    #[cfg(target_os = "linux")]
    khr::xlib_surface::NAME.as_ptr(),
    #[cfg(target_os = "windows")]
    khr::win32_surface::NAME.as_ptr(),
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
    //let features = unsafe { instance.get_physical_device_features(p) };
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

    let features = vk::PhysicalDeviceFeatures::default().fill_mode_non_solid(true);

    let device_create_info = vk::DeviceCreateInfo::default()
      .queue_create_infos(&queue_create_infos)
      .enabled_extension_names(&device_extension_name_ptrs)
      .enabled_features(&features);

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
  depth_image: vk::Image,
  depth_image_allocation: vulkan::Allocation,
  depth_image_view: vk::ImageView,
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
      depth_image,
      depth_image_allocation,
      depth_image_view,
    })
  }

  unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &mut vulkan::Allocator) {
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

  fn create_frame_buffers(
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

fn init_render_pass(
  logical_device: &ash::Device,
  format: vk::Format,
) -> Result<vk::RenderPass, vk::Result> {
  let attachment = [
    vk::AttachmentDescription::default()
      .format(format)
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::STORE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
    vk::AttachmentDescription::default()
      .format(vk::Format::D32_SFLOAT)
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::DONT_CARE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
  ];

  let color_attachment_ref = [vk::AttachmentReference::default()
    .attachment(0)
    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
  let depth_attachment_ref = vk::AttachmentReference::default()
    .attachment(1)
    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

  let subpass = [vk::SubpassDescription::default()
    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    .depth_stencil_attachment(&depth_attachment_ref)
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
  descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
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
        .format(vk::Format::R32G32B32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(1)
        .offset(0)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(2)
        .offset(16)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(3)
        .offset(32)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(4)
        .offset(48)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(5)
        .offset(64)
        .format(vk::Format::R32G32B32_SFLOAT),
    ];

    let vertex_binding_descs = [
      vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(12)
        .input_rate(vk::VertexInputRate::VERTEX),
      vk::VertexInputBindingDescription::default()
        .binding(1)
        .stride(76)
        .input_rate(vk::VertexInputRate::INSTANCE),
    ];

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
      .vertex_binding_descriptions(&vertex_binding_descs)
      .vertex_attribute_descriptions(&vertex_attrib_descs);
    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
      .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

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

    let descriptor_set_layout_binding_descs = [vk::DescriptorSetLayoutBinding::default()
      .binding(0)
      .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
      .descriptor_count(1)
      .stage_flags(vk::ShaderStageFlags::VERTEX)];
    let descriptor_set_layout_create_info =
      vk::DescriptorSetLayoutCreateInfo::default().bindings(&descriptor_set_layout_binding_descs);
    let descriptor_set_layout = unsafe {
      logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
    }?;
    let descriptor_set_layouts = vec![descriptor_set_layout];

    let pipeline_layout_create_info =
      vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_set_layouts);
    let pipeline_layout =
      unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;

    let depth_sencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
      .depth_test_enable(true)
      .depth_write_enable(true)
      .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
      .stages(&shader_stages)
      .vertex_input_state(&vertex_input_info)
      .input_assembly_state(&input_assembly_info)
      .viewport_state(&viewport_info)
      .rasterization_state(&rasterizer_info)
      .multisample_state(&multisample_info)
      .depth_stencil_state(&depth_sencil_info)
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
      descriptor_set_layouts,
    })
  }

  unsafe fn cleanup(&self, logical_device: &ash::Device) {
    for layout in &self.descriptor_set_layouts {
      logical_device.destroy_descriptor_set_layout(*layout, None);
    }
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

struct Buffer {
  buffer: vk::Buffer,
  allocation: vulkan::Allocation,
  size: u64,
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

    Ok(Self {
      buffer,
      allocation,
      size,
    })
  }

  fn fill<T: Sized>(&mut self, data: &[T]) -> Result<(), vk::Result> {
    let bytes_to_write = std::mem::size_of_val(data) as u64;
    if bytes_to_write > self.size {
      return Err(vk::Result::ERROR_OUT_OF_HOST_MEMORY);
    }
    let data_ptr = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
    unsafe {
      data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
    }
    Ok(())
  }
}

struct Model<V, I> {
  vertex_data: Vec<V>,
  handle_to_index: std::collections::HashMap<usize, usize>,
  handles: Vec<usize>,
  instances: Vec<I>,
  first_invisible: usize,
  next_handle: usize,
  vertex_buffer: Option<Buffer>,
  instance_buffer: Option<Buffer>,
}

#[repr(C)]
struct InstanceData {
  model_matrix: [[f32; 4]; 4],
  color: [f32; 3],
}

impl<V, I> Model<V, I> {
  fn get(&self, handle: usize) -> Option<&I> {
    self.instances.get(*self.handle_to_index.get(&handle)?)
  }

  fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
    self.instances.get_mut(*self.handle_to_index.get(&handle)?)
  }

  fn swap_by_handle(&mut self, i: usize, j: usize) -> Result<(), InvalidHandle> {
    if i == j {
      return Ok(());
    }
    let i = *self.handle_to_index.get(&i).ok_or(InvalidHandle)?;
    let j = *self.handle_to_index.get(&j).ok_or(InvalidHandle)?;
    self.swap_by_index(i, j);
    Ok(())
  }

  fn swap_by_index(&mut self, i: usize, j: usize) {
    if i == j {
      return;
    }
    self.instances.swap(i, j);
    self.handles.swap(i, j);
    self.handle_to_index.insert(self.handles[i], i);
    self.handle_to_index.insert(self.handles[j], j);
  }

  fn is_visible(&self, handle: usize) -> Result<bool, InvalidHandle> {
    Ok(*self.handle_to_index.get(&handle).ok_or(InvalidHandle)? < self.first_invisible)
  }

  fn make_visible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
    let index = *self.handle_to_index.get(&handle).ok_or(InvalidHandle)?;
    if index >= self.first_invisible {
      self.swap_by_index(index, self.first_invisible);
      self.first_invisible += 1;
    }
    Ok(())
  }

  fn make_invisible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
    let index = *self.handle_to_index.get(&handle).ok_or(InvalidHandle)?;
    if index < self.first_invisible {
      self.swap_by_index(index, self.first_invisible - 1);
      self.first_invisible -= 1;
    }
    Ok(())
  }

  fn insert(&mut self, instance: I) -> usize {
    let handle = self.next_handle;
    self.next_handle += 1;
    self.handles.push(handle);
    self.instances.push(instance);
    self
      .handle_to_index
      .insert(handle, self.instances.len() - 1);
    handle
  }

  fn insert_visibly(&mut self, instance: I) -> usize {
    let handle = self.insert(instance);
    self.make_visible(handle).ok();
    handle
  }

  fn remove(&mut self, handle: usize) -> Result<I, InvalidHandle> {
    let index = *self.handle_to_index.get(&handle).ok_or(InvalidHandle)?;
    let instance = self.instances.remove(index);
    self.handles.remove(index);
    self.handle_to_index.remove(&handle);
    if index < self.first_invisible {
      self.first_invisible -= 1;
    }
    Ok(instance)
  }

  fn update_vertex_buffer(
    &mut self,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
  ) -> Result<(), vk::Result> {
    let data = self.vertex_data.as_slice();
    if let Some(buffer) = &mut self.vertex_buffer {
      buffer.fill(data)?;
    } else {
      let mut buffer = Buffer::new(
        allocator,
        device,
        std::mem::size_of_val(data) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
      )?;
      buffer.fill(data)?;
      self.vertex_buffer = Some(buffer);
    }
    Ok(())
  }

  fn update_instance_buffer(
    &mut self,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
  ) -> Result<(), vk::Result> {
    let data = &self.instances[..self.first_invisible];
    if let Some(buffer) = &mut self.instance_buffer {
      buffer.fill(data)?;
    } else {
      let mut buffer = Buffer::new(
        allocator,
        device,
        std::mem::size_of_val(data) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
      )?;
      buffer.fill(data)?;
      self.instance_buffer = Some(buffer);
    }
    Ok(())
  }

  fn draw(&self, logical_device: &ash::Device, command_buffer: vk::CommandBuffer) {
    if let Some(vertex_buffer) = &self.vertex_buffer {
      if let Some(instance_buffer) = &self.instance_buffer {
        if self.first_invisible > 0 {
          unsafe {
            logical_device.cmd_bind_vertex_buffers(
              command_buffer,
              0,
              &[vertex_buffer.buffer],
              &[0],
            );
            logical_device.cmd_bind_vertex_buffers(
              command_buffer,
              1,
              &[instance_buffer.buffer],
              &[0],
            );
            logical_device.cmd_draw(
              command_buffer,
              self.vertex_data.len() as u32,
              self.first_invisible as u32,
              0,
              0,
            );
          }
        }
      }
    }
  }

  fn cleanup(&mut self, logical_device: &ash::Device, allocator: &mut vulkan::Allocator) {
    if let Some(buffer) = self.vertex_buffer.take() {
      unsafe {
        logical_device.destroy_buffer(buffer.buffer, None);
        allocator.free(buffer.allocation).unwrap();
      }
    }
    if let Some(buffer) = self.instance_buffer.take() {
      unsafe {
        logical_device.destroy_buffer(buffer.buffer, None);
        allocator.free(buffer.allocation).unwrap();
      }
    }
  }
}

impl Model<[f32; 3], InstanceData> {
  fn cube() -> Model<[f32; 3], InstanceData> {
    let lbf = [-1.0, 1.0, 0.0];
    let lbb = [-1.0, 1.0, 1.0];
    let ltf = [-1.0, -1.0, 0.0];
    let ltb = [-1.0, -1.0, 1.0];
    let rbf = [1.0, 1.0, 0.0];
    let rbb = [1.0, 1.0, 1.0];
    let rtf = [1.0, -1.0, 0.0];
    let rtb = [1.0, -1.0, 1.0];

    Model {
      vertex_data: vec![
        lbf, lbb, rbb, lbf, rbb, rbf, //bottom
        ltf, rtb, ltb, ltf, rtf, rtb, //top
        lbf, rtf, ltf, lbf, rbf, rtf, //front
        lbb, ltb, rtb, lbb, rtb, rbb, //back
        lbf, ltf, lbb, lbb, ltf, ltb, //left
        rbf, rbb, rtf, rbb, rtb, rtf, //right
      ],
      handle_to_index: std::collections::HashMap::new(),
      handles: vec![],
      instances: vec![],
      first_invisible: 0,
      next_handle: 0,
      vertex_buffer: None,
      instance_buffer: None,
    }
  }
}

#[derive(Debug, Clone)]
struct InvalidHandle;

impl std::fmt::Display for InvalidHandle {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Invalid handle")
  }
}

impl std::error::Error for InvalidHandle {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}

struct CameraBuilder {
  position: g::Vec3,
  view_direction: g::Vec3,
  up: g::Vec3,
  fov: f32,
  aspect_ratio: f32,
  near: f32,
  far: f32,
}

impl CameraBuilder {
  fn position(mut self, position: g::Vec3) -> Self {
    self.position = position;
    self
  }

  fn view_direction(mut self, view_direction: g::Vec3) -> Self {
    self.view_direction = view_direction.normalize();
    self
  }

  fn up(mut self, up: g::Vec3) -> Self {
    self.up = up.normalize();
    self
  }

  fn fov(mut self, fov: f32) -> Self {
    self.fov = fov.max(0.01).min(std::f32::consts::PI - 0.01);
    self
  }

  fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
    self.aspect_ratio = aspect_ratio;
    self
  }

  fn near(mut self, near: f32) -> Self {
    self.near = near;
    self
  }

  fn far(mut self, far: f32) -> Self {
    self.far = far;
    self
  }

  fn build(self) -> Camera {
    let mut cam = Camera {
      view_matrix: g::Mat4::IDENTITY,
      position: self.position,
      view_direction: self.view_direction,
      up: (self.up - self.view_direction.dot(self.view_direction * self.view_direction)).normalize(),
      fov: self.fov,
      aspect_ratio: self.aspect_ratio,
      near: self.near,
      far: self.far,
      projection_matrix: g::Mat4::IDENTITY,
    };
    cam.update_projection_matrix();
    cam.update_view_matrix();
    cam
  }
}

struct Camera {
  view_matrix: g::Mat4,
  position: g::Vec3,
  view_direction: g::Vec3,
  up: g::Vec3,
  fov: f32,
  aspect_ratio: f32,
  near: f32,
  far: f32,
  projection_matrix: g::Mat4,
}

impl Camera {
  fn builder() -> CameraBuilder {
    CameraBuilder {
      position: g::Vec3::new(0.0, 3.0, -3.0),
      view_direction: g::Vec3::new(0.0, -1.0, 1.0),
      up: g::Vec3::new(0.0, 1.0, 1.0),
      fov: std::f32::consts::FRAC_PI_3,
      aspect_ratio: 800.0 / 600.0,
      near: 0.1,
      far: 100.0,
    }
  }

  fn update_buffer(&self, buffer: &mut Buffer) -> Result<(), vk::Result> {
    let data = [self.view_matrix.to_cols_array_2d(), self.projection_matrix.to_cols_array_2d()];
    buffer.fill(&data)?;
    Ok(())
  }

  fn update_view_matrix(&mut self) {
    self.view_matrix = g::Mat4::look_at_rh(self.position, self.position + self.view_direction, -self.up);
  }

  fn update_projection_matrix(&mut self) {
    self.projection_matrix = g::Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
  }

  fn move_forward(&mut self, amount: f32) {
    self.position += self.view_direction * amount;
    self.update_view_matrix();
  }

  fn move_backward(&mut self, amount: f32) {
    self.move_forward(-amount);
  }

  fn move_right(&mut self, amount: f32) {
    self.position += self.view_direction.cross(-self.up).normalize() * amount;
    self.update_view_matrix();
  }

  fn move_left(&mut self, amount: f32) {
    self.move_right(-amount);
  }

  fn move_up(&mut self, amount: f32) {
    self.position += self.up * amount;
    self.update_view_matrix();
  }

  fn move_down(&mut self, amount: f32) {
    self.move_up(-amount);
  }

  fn turn_right(&mut self, amount: f32) {
    let rotation = g::Quat::from_axis_angle(self.up, amount);
    self.view_direction = rotation * self.view_direction;
    self.update_view_matrix();
  }

  fn turn_left(&mut self, amount: f32) {
    self.turn_right(-amount);
  }

  fn turn_up(&mut self, amount: f32) {
    let rotation = g::Quat::from_axis_angle(self.view_direction.cross(self.up), amount);
    self.view_direction = rotation * self.view_direction;
    self.up = rotation * self.up;
    self.update_view_matrix();
  }

  fn turn_down(&mut self, amount: f32) {
    self.turn_up(-amount);
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
  models: Vec<Model<[f32; 3], InstanceData>>,
  uniform_buffer: Buffer,
  descriptor_pool: vk::DescriptorPool,
  descriptor_sets: Vec<vk::DescriptorSet>,
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
    let camera_transform = [g::Mat4::IDENTITY.to_cols_array_2d(), g::Mat4::IDENTITY.to_cols_array_2d()];
    uniform_buffer.fill(&camera_transform)?;

    let pool_sizes = [vk::DescriptorPoolSize::default()
      .ty(vk::DescriptorType::UNIFORM_BUFFER)
      .descriptor_count(swapchain_dong.amount_of_images)];
    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::default()
      .max_sets(swapchain_dong.amount_of_images)
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
      models: vec![],
      uniform_buffer,
      descriptor_pool,
      descriptor_sets,
    })
  }

  fn update_command_buffer(&mut self, index: usize) -> Result<(), vk::Result> {
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
          float32: [0.0, 0.0, 0.08, 1.0],
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
        &[self.descriptor_sets[index]],
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
}

impl Drop for Aetna {
  fn drop(&mut self) {
    unsafe {
      self
        .device
        .device_wait_idle()
        .expect("Unable to wait for device idle");
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
