use std::mem::ManuallyDrop;

use anyhow::Error;
use config::VulkanConfig;
use debug::Debugger;
use device::Device;
use gpu_allocator::vulkan;
use graphics::Renderer;
use instance::{InstanceDevice, InstanceDeviceConfig};
use surface::Surface;
use winit::window::Window;

mod debug;
mod device;
pub(crate) mod error;
mod graphics;
mod instance;
mod surface;
pub(crate) mod config;

pub(crate) struct Vulkan {
  #[allow(dead_code)]
  entry: ash::Entry,
  debugger: Option<Debugger>,
  instance: InstanceDevice,
  surface: Surface,
  device: Device,
  renderer: Renderer,
  allocator: ManuallyDrop<vulkan::Allocator>,
}

impl Vulkan {
  pub(crate) fn init(mut config: VulkanConfig, window: &Window) -> Result<Self, Error> {
    let entry = unsafe { ash::Entry::load() }?;

    let debugger_info = if config.engine.debug {
      Some(Debugger::init_info(&mut config.engine))
    } else {
      None
    };

    let mut instance_config = InstanceDeviceConfig::default()
      .add_layers(config.engine.layers.clone())
      .add_extensions(config.engine.instance_extensions.clone())
      .add_instance_nexts(std::mem::take(&mut config.engine.instance_next));

    let instance = InstanceDevice::init(&mut instance_config, &entry, &config.app)?;

    let debugger = if config.engine.debug {
      Some(Debugger::init(
        &entry,
        instance.get_instance(),
        debugger_info.unwrap(),
      )?)
    } else {
      None
    };

    let surface = Surface::init(&entry, instance.get_instance(), window)?;
    let device = Device::init(
      instance.get_instance(),
      instance.get_physical_device(),
      &surface,
      &config.engine,
    )?;

    let mut allocator = vulkan::Allocator::new(&vulkan::AllocatorCreateDesc {
      device: device.get_device().clone(),
      physical_device: instance.get_physical_device(),
      instance: instance.get_instance().clone(),
      debug_settings: Default::default(),
      buffer_device_address: false,
      allocation_sizes: Default::default(),
    })?;

    let renderer = Renderer::init(&instance, &device, &mut allocator, &surface, &mut config)?;

    Ok(Vulkan {
      entry,
      debugger,
      instance,
      surface,
      device,
      renderer,
      allocator: ManuallyDrop::new(allocator),
    })
  }

  pub(crate) fn destroy(&mut self) {
    self.renderer.destroy(self.device.get_device(), &mut self.allocator);
    unsafe {
      ManuallyDrop::drop(&mut self.allocator);
    }
    self.device.destroy();
    self.surface.destroy();
    if let Some(debugger) = &mut self.debugger {
      debugger.destroy();
    }
    self.instance.destroy();
  }
}
