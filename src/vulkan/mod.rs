use anyhow::Error;
use ash::vk;
use debug::Debugger;
use instance::{InstanceDevice, InstanceDeviceConfig};
use surface::Surface;
use winit::window::Window;
use device::Device;

use crate::utils::LogLevel;

mod debug;
mod device;
mod error;
mod instance;
mod surface;

pub(crate) struct Vulkan {
  #[allow(dead_code)]
  entry: ash::Entry,
  debugger: Option<Debugger>,
  instance: InstanceDevice,
  surface: Surface,
  device: Device,
}

impl Vulkan {
  pub(crate) fn init(mut config: VulkanConfig, window: &Window) -> Result<Self, Error> {
    let entry = unsafe { ash::Entry::load() }?;

    let debugger_info = if config.debug {
      Some(Debugger::init_info(&mut config))
    } else {
      None
    };

    let mut instance_config = InstanceDeviceConfig::default()
      .add_layers(config.layers.clone())
      .add_extensions(config.instance_extensions.clone())
      .add_instance_nexts(std::mem::take(&mut config.instance_next));

    let instance = InstanceDevice::init(&mut instance_config, &entry)?;

    let debugger = if config.debug {
      Some(Debugger::init(
        &entry,
        instance.get_instance(),
        debugger_info.unwrap(),
      )?)
    } else {
      None
    };

    let surface = Surface::init(&entry, instance.get_instance(), window)?;
    let device = Device::init(instance.get_instance(), instance.get_physical_device(), &surface, &config)?;

    Ok(Vulkan {
      entry,
      debugger,
      instance,
      surface,
      device,
    })
  }

  pub(crate) fn destroy(&mut self) {
    self.device.destroy();
    self.surface.destroy();
    if let Some(debugger) = &mut self.debugger {
      debugger.destroy();
    }
    self.instance.destroy();
  }
}

#[derive(Default)]
pub(crate) struct VulkanConfig<'a> {
  layers: Vec<&'a std::ffi::CStr>,
  instance_extensions: Vec<&'a std::ffi::CStr>,
  instance_next: Vec<Box<dyn vk::ExtendsInstanceCreateInfo>>,
  device_extensions: Vec<&'a std::ffi::CStr>,
  device_features: vk::PhysicalDeviceFeatures,
  debug: bool,
  debug_log_level: vk::DebugUtilsMessageSeverityFlagsEXT,
}

impl<'a> VulkanConfig<'a> {
  pub(crate) fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }

  pub(crate) fn set_debug(mut self, debug: bool) -> Self {
    self.debug = debug;
    self
  }

  pub(crate) fn set_debug_log_level(mut self, level: LogLevel) -> Self {
    self.debug_log_level = match level {
      LogLevel::Verbose => {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
          | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
          | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      }
      LogLevel::Info => {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO
          | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      }
      LogLevel::Warning => {
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      }
      LogLevel::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
      LogLevel::None => vk::DebugUtilsMessageSeverityFlagsEXT::empty(),
    };
    self
  }
}
