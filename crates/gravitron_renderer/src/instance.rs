use anyhow::Error;
use ash::{khr, vk};

use crate::config::VulkanConfig;

const REQUIRED_EXTENSION_NAMES: [*const i8; 1] = [khr::surface::NAME.as_ptr()];

#[cfg(target_os = "linux")]
const REQUIRED_PLATFORM_EXTENSION_NAMES: [*const i8; 0] = [];

#[cfg(target_os = "windows")]
const REQUIRED_PLATFORM_EXTENSION_NAMES: [*const i8; 1] = [khr::win32_surface::NAME.as_ptr()];

#[cfg(target_os = "macos")]
const REQUIRED_PLATFORM_EXTENSION_NAMES: [*const i8; 1] = [ash::mvk::macos_surface::NAME.as_ptr()];

pub struct InstanceDevice {
  instance: ash::Instance,
  physical_device: vk::PhysicalDevice,
}

impl InstanceDevice {
  pub fn init(
    config: &mut InstanceDeviceConfig,
    entry: &ash::Entry,
    vulkan_config: &VulkanConfig,
    #[cfg(target_os = "linux")] is_wayland: bool,
  ) -> Result<Self, Error> {
    let instance = InstanceDevice::init_instance(
      entry,
      config,
      vulkan_config,
      #[cfg(target_os = "linux")]
      is_wayland,
    )?;
    let (physical_device, _) = InstanceDevice::init_physical_device_and_properties(&instance)?;

    Ok(Self {
      instance,
      physical_device,
    })
  }

  pub fn get_instance(&self) -> &ash::Instance {
    &self.instance
  }

  pub fn get_physical_device(&self) -> vk::PhysicalDevice {
    self.physical_device
  }

  fn init_instance(
    entry: &ash::Entry,
    config: &mut InstanceDeviceConfig,
    vulkan_config: &VulkanConfig,
    #[cfg(target_os = "linux")] is_wayland: bool,
  ) -> Result<ash::Instance, Error> {
    let engine_name = std::ffi::CString::new("Vulkan Game Engine")?;
    let app_name = std::ffi::CString::new(vulkan_config.title.clone())?;

    let app_info = vk::ApplicationInfo::default()
      .application_name(&app_name)
      .engine_name(&engine_name)
      .engine_version(vk::make_api_version(0, 0, 1, 0))
      .application_version(vulkan_config.version)
      .api_version(vk::make_api_version(0, 1, 3, 278));

    let layer_name_ptrs: Vec<*const i8> = config
      .layer_names
      .iter()
      .map(|layer_name| layer_name.as_ptr())
      .collect();

    let mut extension_name_ptrs: Vec<*const i8> = config
      .extension_names
      .iter()
      .map(|extension_name| extension_name.as_ptr())
      .collect();
    extension_name_ptrs.extend(REQUIRED_EXTENSION_NAMES.iter());
    extension_name_ptrs.extend(REQUIRED_PLATFORM_EXTENSION_NAMES.iter());

    #[cfg(target_os = "linux")]
    if is_wayland {
      extension_name_ptrs.push(khr::wayland_surface::NAME.as_ptr());
    } else {
      extension_name_ptrs.push(khr::xlib_surface::NAME.as_ptr());
    }

    let mut instance_create_info = vk::InstanceCreateInfo::default()
      .application_info(&app_info)
      .enabled_layer_names(&layer_name_ptrs)
      .enabled_extension_names(&extension_name_ptrs);

    for info in &mut config.instance_next {
      instance_create_info = instance_create_info.push_next(info.as_mut());
    }

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
      } else if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
        physical_device = Some((p, properties));
      }
    }
    Ok(physical_device.unwrap())
  }

  pub fn destroy(&self) {
    unsafe {
      self.instance.destroy_instance(None);
    }
  }
}

#[derive(Default)]
pub struct InstanceDeviceConfig<'a> {
  layer_names: Vec<&'a std::ffi::CStr>,
  extension_names: Vec<&'a std::ffi::CStr>,
  instance_next: Vec<Box<dyn vk::ExtendsInstanceCreateInfo + Send>>,
}

impl<'a> InstanceDeviceConfig<'a> {
  pub fn add_layers(mut self, layers: Vec<&'a std::ffi::CStr>) -> Self {
    for layer in layers {
      self.layer_names.push(layer);
    }
    self
  }

  pub fn add_extensions(mut self, extensions: Vec<&'a std::ffi::CStr>) -> Self {
    for extension in extensions {
      self.extension_names.push(extension);
    }
    self
  }

  pub fn add_instance_nexts(
    mut self,
    nexts: Vec<Box<dyn vk::ExtendsInstanceCreateInfo + Send>>,
  ) -> Self {
    for next in nexts {
      self.instance_next.push(next);
    }
    self
  }
}
