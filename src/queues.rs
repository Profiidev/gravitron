
use ash::{ext, khr, vk};

use crate::surface::SurfaceDong;

pub fn init_instance(
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

pub fn init_physical_device_and_properties(
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

pub struct QueueFamilies {
  pub graphics_q_index: Option<u32>,
  pub transfer_q_index: Option<u32>,
}

impl QueueFamilies {
  pub fn init(
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

pub struct Queues {
  pub graphics: vk::Queue,
  #[allow(dead_code)]
  pub transfer: vk::Queue,
}

impl Queues {
  pub fn init(
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