use anyhow::Error;
use ash::{khr, vk};

const REQUIRED_EXTENSION_NAMES: [*const i8; 1] = [khr::surface::NAME.as_ptr()];

#[cfg(target_os = "linux")]
const REQUIRED_PLATFORM_EXTENSION_NAMES: [*const i8; 2] = [
  khr::wayland_surface::NAME.as_ptr(),
  khr::xlib_surface::NAME.as_ptr(),
];

#[cfg(target_os = "windows")]
const REQUIRED_PLATFORM_EXTENSION_NAMES: [*const i8; 1] = [khr::win32_surface::NAME.as_ptr()];

pub(crate) struct InstanceDevice {
  instance: ash::Instance,
  physical_device: vk::PhysicalDevice,
}

impl InstanceDevice {
  pub(crate) fn init(config: &mut InstanceDeviceConfig, entry: &ash::Entry) -> Result<Self, Error> {
    let instance = InstanceDevice::init_instance(entry, config)?;
    let (physical_device, _) = InstanceDevice::init_physical_device_and_properties(&instance)?;

    Ok(Self {
      instance,
      physical_device,
    })
  }

  pub(crate) fn get_instance(&self) -> &ash::Instance {
    &self.instance
  }

  pub(crate) fn get_physical_device(&self) -> vk::PhysicalDevice {
    self.physical_device
  }

  fn init_instance(
    entry: &ash::Entry,
    config: &mut InstanceDeviceConfig,
  ) -> Result<ash::Instance, Error> {
    let engine_name = std::ffi::CString::new("Vulkan Engine")?;
    let app_name = std::ffi::CString::new("Test App")?;

    let app_info = vk::ApplicationInfo::default()
      .application_name(&app_name)
      .engine_name(&engine_name)
      .engine_version(vk::make_api_version(0, 0, 42, 0))
      .application_version(vk::make_api_version(0, 0, 1, 0))
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

  pub(crate) fn destroy(&self) {
    unsafe {
      self.instance.destroy_instance(None);
    }
  }
}

#[derive(Default)]
pub(crate) struct InstanceDeviceConfig<'a> {
  layer_names: Vec<&'a std::ffi::CStr>,
  extension_names: Vec<&'a std::ffi::CStr>,
  instance_next: Vec<Box<dyn vk::ExtendsInstanceCreateInfo>>,
}

impl<'a> InstanceDeviceConfig<'a> {
  pub(crate) fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layer_names.push(layer);
    self
  }

  pub(crate) fn add_layers(mut self, layers: Vec<&'a std::ffi::CStr>) -> Self {
    for layer in layers {
      self.layer_names.push(layer);
    }
    self
  }

  pub(crate) fn add_extension(mut self, extension: &'a std::ffi::CStr) -> Self {
    self.extension_names.push(extension);
    self
  }

  pub(crate) fn add_extensions(mut self, extensions: Vec<&'a std::ffi::CStr>) -> Self {
    for extension in extensions {
      self.extension_names.push(extension);
    }
    self
  }

  pub(crate) fn add_instance_next(mut self, next: Box<dyn vk::ExtendsInstanceCreateInfo>) -> Self {
    self.instance_next.push(next);
    self
  }

  pub(crate) fn add_instance_nexts(
    mut self,
    nexts: Vec<Box<dyn vk::ExtendsInstanceCreateInfo>>,
  ) -> Self {
    for next in nexts {
      self.instance_next.push(next);
    }
    self
  }
}
