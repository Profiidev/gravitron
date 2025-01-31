use anyhow::Error;
use gravitron_plugin::{
  app::{App, AppBuilder, Cleanup, Finalize},
  config::{vulkan::VulkanConfig, AppConfig},
};
use gravitron_window::ecs::resources::handle::WindowHandle;
use memory::MemoryManager;
use model::ModelManager;

#[cfg(feature = "debug")]
use crate::debug::Debugger;

use crate::{
  device::Device,
  graphics::Renderer,
  instance::{InstanceDevice, InstanceDeviceConfig},
  pipeline::{manager::PipelineManager, pools::Pools},
  surface::Surface,
};

pub mod memory {
  pub use crate::memory::*;
}

pub mod model {
  pub use crate::model::*;
}

pub(crate) struct Resources {
  memory_manager: MemoryManager,
  model_manager: ModelManager,
}

impl Resources {
  pub fn add_resources(self, builder: &mut AppBuilder<Finalize>) {
    builder.add_resource(self.memory_manager);
    builder.add_resource(self.model_manager);
  }

  pub(crate) fn create(
    mut config: VulkanConfig,
    app_config: &AppConfig,
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

    let mut renderer = Renderer::init(
      &instance,
      &device,
      &mut memory_manager,
      &surface,
      &mut config,
      &app_config.window,
      &mut pools,
    )?;

    let pipeline_manager = PipelineManager::init(
      device.get_device(),
      renderer.render_pass(),
      renderer.light_render_pass(),
      renderer.swapchain(),
      &mut config.shaders,
      config.textures,
      &mut memory_manager,
    )?;

    renderer.record_command_buffer(&pipeline_manager, &mut memory_manager)?;

    Ok(Resources {
      memory_manager,
      model_manager,
    })
  }
}

pub(crate) fn cleanup_resource(app: &mut App<Cleanup>) -> Result<(), Error> {
  app
    .get_resource_mut::<MemoryManager>()
    .expect("Failed get MemoryManager")
    .cleanup()?;

  Ok(())
}
