use std::collections::HashMap;

use anyhow::Error;
#[cfg(feature = "debug")]
use debug::Debugger;
use device::Device;
use graphics::{
  resources::{
    lighting::{LightInfo, PointLight, SpotLight},
    model::{InstanceData, ModelId},
  },
  Renderer,
};
use instance::{InstanceDevice, InstanceDeviceConfig};
use memory::{manager::MemoryManager, BufferMemory};
use pipeline::manager::PipelineManager;
use pipeline::pools::Pools;
use surface::Surface;
use winit::window::Window;

use crate::{
  config::{app::AppConfig, vulkan::VulkanConfig},
  ecs::components::camera::Camera,
};

pub use vk_shader_macros::{glsl, include_glsl};

#[cfg(feature = "debug")]
mod debug;
mod device;
pub mod error;
pub mod graphics;
mod instance;
pub mod memory;
mod pipeline;
mod surface;

pub struct Vulkan {
  #[allow(dead_code)]
  entry: ash::Entry,
  #[cfg(feature = "debug")]
  debugger: Debugger,
  instance: InstanceDevice,
  surface: Surface,
  device: Device,
  renderer: Renderer,
  pools: Pools,
  memory_manager: MemoryManager,
  pipeline_manager: PipelineManager,
}

impl Vulkan {
  pub fn init(
    mut config: VulkanConfig,
    app_config: &AppConfig,
    window: &Window,
    #[cfg(target_os = "linux")] is_wayland: bool,
  ) -> Result<Self, Error> {
    let entry = unsafe { ash::Entry::load() }?;

    #[cfg(feature = "debug")]
    let debugger_info = Debugger::init_info(&mut config.renderer);

    let mut instance_config = InstanceDeviceConfig::default()
      .add_layers(config.renderer.layers.clone())
      .add_extensions(config.renderer.instance_extensions.clone())
      .add_instance_nexts(std::mem::take(&mut config.renderer.instance_next));

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

    let mut renderer = Renderer::init(
      &instance,
      &device,
      &mut memory_manager,
      &surface,
      &mut config,
      app_config,
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

    Ok(Vulkan {
      entry,
      #[cfg(feature = "debug")]
      debugger,
      instance,
      surface,
      device,
      renderer,
      memory_manager,
      pools,
      pipeline_manager,
    })
  }

  pub fn wait_for_draw_start(&self) {
    let device = self.device.get_device();
    self.renderer.wait_for_draw_start(device);
  }

  pub fn draw_frame(&mut self) {
    self.renderer.draw_frame(&self.device);
  }

  pub fn update_descriptor<T: Sized>(&mut self, mem: &BufferMemory, data: &[T]) {
    assert!(mem.size() >= std::mem::size_of_val(data));
    self
      .pipeline_manager
      .update_descriptor(&mut self.memory_manager, mem, data);
  }

  pub fn get_descriptor_mem(
    &mut self,
    pipeline_name: &str,
    descriptor_set: usize,
    descriptor: usize,
  ) -> Option<BufferMemory> {
    self
      .pipeline_manager
      .get_descriptor_mem(pipeline_name, descriptor_set, descriptor)
  }

  pub fn update_draw_info(
    &mut self,
    instances: HashMap<ModelId, HashMap<String, Vec<InstanceData>>>,
    light_info: LightInfo,
    pls: &[PointLight],
    sls: &[SpotLight],
  ) {
    let resized = self
      .pipeline_manager
      .update_lights(&mut self.memory_manager, light_info, pls, sls)
      .expect("Error while updating lights");
    self
      .renderer
      .update_draw_buffer(&mut self.memory_manager, instances, resized);
    self
      .renderer
      .record_command_buffer(&self.pipeline_manager, &mut self.memory_manager)
      .expect("Command Buffer Error");
  }

  pub fn update_camera(&mut self, camera: &Camera) {
    self
      .pipeline_manager
      .update_camera(&mut self.memory_manager, camera);
  }

  pub fn destroy(&mut self) {
    unsafe {
      self
        .device
        .get_device()
        .device_wait_idle()
        .expect("Unable to wait for device idle");
    }

    self.pipeline_manager.destroy();
    self.renderer.destroy();
    self.memory_manager.cleanup().unwrap();
    unsafe {
      self.pools.cleanup();
    }

    self.device.destroy();
    self.surface.destroy();

    #[cfg(feature = "debug")]
    self.debugger.destroy();

    self.instance.destroy();
  }
}
