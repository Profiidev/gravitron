use std::mem::ManuallyDrop;

use anyhow::Error;
use debug::Debugger;
use device::Device;
use gpu_allocator::vulkan;
use graphics::Renderer;
use instance::{InstanceDevice, InstanceDeviceConfig};
use surface::Surface;
use winit::window::Window;

use crate::config::{app::AppConfig, vulkan::VulkanConfig};

#[cfg(feature = "debug")]
mod debug;
mod device;
pub mod error;
mod graphics;
mod instance;
mod surface;

pub struct Vulkan {
  #[allow(dead_code)]
  entry: ash::Entry,
  #[cfg(feature = "debug")]
  debugger: Debugger,
  instance: InstanceDevice,
  window: Window,
  surface: Surface,
  device: Device,
  renderer: Renderer,
  allocator: ManuallyDrop<vulkan::Allocator>,
}

impl Vulkan {
  pub fn init(
    mut config: VulkanConfig,
    app_config: &AppConfig,
    window: Window,
  ) -> Result<Self, Error> {
    let entry = unsafe { ash::Entry::load() }?;

    #[cfg(feature = "debug")]
    let debugger_info = Debugger::init_info(&mut config.renderer);

    let mut instance_config = InstanceDeviceConfig::default()
      .add_layers(config.renderer.layers.clone())
      .add_extensions(config.renderer.instance_extensions.clone())
      .add_instance_nexts(std::mem::take(&mut config.renderer.instance_next));

    let instance = InstanceDevice::init(&mut instance_config, &entry, app_config)?;

    #[cfg(feature = "debug")]
    let debugger = Debugger::init(&entry, instance.get_instance(), debugger_info)?;

    let surface = Surface::init(&entry, instance.get_instance(), &window)?;
    let device = Device::init(
      instance.get_instance(),
      instance.get_physical_device(),
      &surface,
      &config.renderer,
    )?;

    let mut allocator = vulkan::Allocator::new(&vulkan::AllocatorCreateDesc {
      device: device.get_device().clone(),
      physical_device: instance.get_physical_device(),
      instance: instance.get_instance().clone(),
      debug_settings: Default::default(),
      buffer_device_address: false,
      allocation_sizes: Default::default(),
    })?;

    let renderer = Renderer::init(
      &instance,
      &device,
      &mut allocator,
      &surface,
      &mut config,
      app_config,
    )?;

    Ok(Vulkan {
      entry,
      #[cfg(feature = "debug")]
      debugger,
      instance,
      window,
      surface,
      device,
      renderer,
      allocator: ManuallyDrop::new(allocator),
    })
  }

  pub fn wait_for_draw_start(&self) {
    let device = self.device.get_device();
    self.renderer.get_swapchain().wait_for_draw_start(device);
  }

  pub fn draw_frame(&mut self) {
    self.renderer.get_swapchain_mut().draw_frame(&self.device);
  }

  pub fn testing(&self) {
    self
      .renderer
      .testing(self.device.get_device())
      .expect("Command Buffer Error");
  }

  pub fn destroy(&mut self) {
    self
      .renderer
      .destroy(self.device.get_device(), &mut self.allocator);
    unsafe {
      ManuallyDrop::drop(&mut self.allocator);
    }
    self.device.destroy();
    self.surface.destroy();

    #[cfg(feature = "debug")]
    self.debugger.destroy();

    self.instance.destroy();
  }
}
