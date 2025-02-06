use anyhow::Error;
use gravitron_plugin::{
  app::{App, AppBuilder, Cleanup, Finalize},
  config::AppConfig,
};
use gravitron_window::{config::WindowConfig, ecs::resources::handle::WindowHandle};
use memory::MemoryManager;
use model::ModelManager;
use pipeline::DescriptorManager;

#[cfg(feature = "debug")]
use crate::debug::Debugger;

use crate::{
  config::VulkanConfig,
  device::Device,
  instance::{InstanceDevice, InstanceDeviceConfig},
  pipeline::{manager::PipelineManager, pools::Pools},
  renderer::Renderer,
  surface::Surface,
};

pub mod memory {
  pub use crate::memory::*;
}

pub mod model {
  pub use crate::model::*;
}

pub mod pipeline {
  pub use crate::pipeline::*;
}

pub mod renderer {
  pub use crate::renderer::*;
}

struct Vulkan {
  #[allow(dead_code)]
  entry: ash::Entry,
  #[cfg(feature = "debug")]
  debugger: Debugger,
  instance: InstanceDevice,
  surface: Surface,
  device: Device,
  pools: Pools,
}

impl Vulkan {
  fn wait_for_idle(&self) {
    unsafe {
      self
        .device
        .get_device()
        .device_wait_idle()
        .expect("Unable to wait for device idle");
    }
  }

  fn cleanup(&self) {
    unsafe {
      self.pools.cleanup();
    }
    self.device.cleanup();
    self.surface.cleanup();
    #[cfg(feature = "debug")]
    self.debugger.cleanup();
    self.instance.cleanup();
  }
}

pub(crate) struct Resources {
  memory_manager: MemoryManager,
  model_manager: ModelManager,
  descriptor_manager: DescriptorManager,
  pipeline_manager: PipelineManager,
  renderer: Renderer,
  vulkan: Vulkan,
}

impl Resources {
  pub fn add_resources(self, builder: &mut AppBuilder<Finalize>) {
    builder.add_resource(self.memory_manager);
    builder.add_resource(self.model_manager);
    builder.add_resource(self.descriptor_manager);
    builder.add_resource(self.pipeline_manager);
    builder.add_resource(self.renderer);
    builder.add_resource(self.vulkan);
  }

  pub(crate) fn create(
    mut config: VulkanConfig,
    app_config: &AppConfig,
    window_config: &WindowConfig,
    window: &WindowHandle,
    #[cfg(target_os = "linux")] is_wayland: bool,
  ) -> Result<Self, Error> {
    let entry = unsafe { ash::Entry::load() }?;

    #[cfg(feature = "debug")]
    let mut instance_next = Vec::new();
    #[cfg(not(feature = "debug"))]
    let instance_next = Vec::new();
    #[cfg(feature = "debug")]
    let debugger_info = Debugger::init_info(&mut config.renderer, &mut instance_next);

    let mut instance_config = InstanceDeviceConfig::default()
      .add_layers(config.renderer.layers.clone())
      .add_extensions(config.renderer.instance_extensions.clone())
      .add_instance_nexts(instance_next);

    let instance = InstanceDevice::init(
      &mut instance_config,
      &entry,
      app_config,
      window_config,
      #[cfg(target_os = "linux")]
      is_wayland,
    )?;

    #[cfg(feature = "debug")]
    let debugger = Debugger::init(&entry, instance.get_instance(), debugger_info)?;

    let surface = Surface::init(&entry, instance.get_instance(), window)?;
    let device = Device::init(
      instance.get_instance(),
      instance.get_physical_device(),
      &surface,
      &config.renderer,
    )?;

    let mut pools = Pools::init(device.get_device(), device.get_queue_families())?;

    let mut memory_manager = MemoryManager::new(&instance, &device, &mut pools)?;
    let model_manager = ModelManager::new(&mut memory_manager)?;
    let mut descriptor_manager = DescriptorManager::new(device.get_device())?;

    let (mut renderer, pipeline_manager) = Renderer::init(
      &instance,
      &device,
      &mut memory_manager,
      &mut descriptor_manager,
      &surface,
      window_config,
      &mut pools,
    )?;

    renderer.record_command_buffer(
      &pipeline_manager,
      &descriptor_manager,
      &mut memory_manager,
      &model_manager,
    )?;

    let vulkan = Vulkan {
      entry,
      #[cfg(feature = "debug")]
      debugger,
      instance,
      surface,
      device,
      pools,
    };

    Ok(Resources {
      memory_manager,
      model_manager,
      descriptor_manager,
      pipeline_manager,
      renderer,
      vulkan,
    })
  }
}

pub(crate) fn cleanup_resource(app: &mut App<Cleanup>) -> Result<(), Error> {
  app
    .get_resource_mut::<Vulkan>()
    .expect("Failed get Vulkan")
    .wait_for_idle();
  app
    .get_resource_mut::<Renderer>()
    .expect("Failed get Renderer")
    .cleanup();
  app
    .get_resource_mut::<PipelineManager>()
    .expect("Failed get PipelineManager")
    .cleanup();
  app
    .get_resource_mut::<DescriptorManager>()
    .expect("Failed get DescriptorManager")
    .cleanup();
  app
    .get_resource_mut::<MemoryManager>()
    .expect("Failed get MemoryManager")
    .cleanup()?;
  app
    .get_resource_mut::<Vulkan>()
    .expect("Failed get Vulkan")
    .cleanup();

  Ok(())
}
