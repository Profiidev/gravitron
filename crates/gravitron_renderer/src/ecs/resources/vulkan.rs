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
  pipeline_manager: PipelineManager,
}

impl Vulkan {
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
