use anyhow::Error;
use ash::{ext, vk};
use debug::{Debugger, VALIDATION_LAYER};
use instance::{InstanceDevice, InstanceDeviceConfig};

mod debug;
mod error;
mod instance;

pub(crate) struct Vulkan {
  debugger: Option<Debugger>,
  instance: InstanceDevice,
}

impl Vulkan {
  pub(crate) fn init(mut config: VulkanConfig) -> Result<Self, Error> {
    let entry = unsafe { ash::Entry::load() }?;

    let mut instance_info_next: Vec<Box<dyn vk::ExtendsInstanceCreateInfo>> = Vec::new();
    let mut instance_extensions = Vec::new();

    let debugger_info = if config.debug {
      config
        .layers
        .push(VALIDATION_LAYER);

      let validation_ext = vk::ValidationFeaturesEXT::default()
        .enabled_validation_features(&[vk::ValidationFeatureEnableEXT::DEBUG_PRINTF]);
      instance_info_next.push(Box::new(validation_ext));

      instance_extensions.push(ext::debug_report::NAME);
      instance_extensions.push(ext::debug_utils::NAME);

      let mut debugger_info = Debugger::info();
      instance_info_next.append(&mut debugger_info.instance_next());
      Some(debugger_info)
    } else {
      None
    };

    let mut instance_config = InstanceDeviceConfig::default()
      .add_layers(config.layers)
      .add_extensions(instance_extensions)
      .add_instance_nexts(instance_info_next);

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

    Ok(Vulkan { debugger, instance })
  }

  pub(crate) fn destroy(&mut self) {
    if let Some(debugger) = &mut self.debugger {
      debugger.destroy();
    }
    self.instance.destroy();
  }
}

#[derive(Debug, Default)]
pub(crate) struct VulkanConfig<'a> {
  layers: Vec<&'a std::ffi::CStr>,
  debug: bool,
}

impl<'a> VulkanConfig<'a> {
  pub(crate) fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }

  pub(crate) fn add_layers(mut self, layers: Vec<&'a std::ffi::CStr>) -> Self {
    for layer in layers {
      self.layers.push(layer);
    }
    self
  }

  pub(crate) fn set_debug(mut self, debug: bool) -> Self {
    self.debug = debug;
    self
  }
}
