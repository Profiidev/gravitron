use ash::vk;

pub use vk::ShaderStageFlags;

#[derive(Default)]
pub struct VulkanConfig {
  pub renderer: RendererConfig<'static>,
  pub shaders: Vec<PipelineType>,
}

impl VulkanConfig {
  pub fn set_renderer_config(mut self, engine: RendererConfig<'static>) -> Self {
    self.renderer = engine;
    self
  }

  pub fn add_graphics_pipeline(mut self, pipeline: GraphicsPipelineConfig) -> Self {
    self.shaders.push(PipelineType::Graphics(pipeline));
    self
  }

  pub fn add_compute_pipeline(mut self, pipeline: ComputePipelineConfig) -> Self {
    self.shaders.push(PipelineType::Compute(pipeline));
    self
  }
}

#[derive(Default)]
pub struct RendererConfig<'a> {
  pub layers: Vec<&'a std::ffi::CStr>,
  pub instance_extensions: Vec<&'a std::ffi::CStr>,
  pub instance_next: Vec<Box<dyn vk::ExtendsInstanceCreateInfo + Send>>,
  pub device_extensions: Vec<&'a std::ffi::CStr>,
  pub device_features: vk::PhysicalDeviceFeatures,
}

impl<'a> RendererConfig<'a> {
  pub fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }
}

pub enum PipelineType {
  Graphics(GraphicsPipelineConfig),
  Compute(ComputePipelineConfig),
}

pub struct GraphicsPipelineConfig {
  pub name: String,
  pub geo_shader: Option<Vec<u32>>,
  pub frag_shader: Vec<u32>,
  pub descriptor_sets: Vec<DescriptorSet>,
}

impl GraphicsPipelineConfig {
  pub fn new(name: String) -> Self {
    Self {
      name,
      geo_shader: None,
      frag_shader: Vec::new(),
      descriptor_sets: Vec::new(),
    }
  }

  pub fn set_geo_shader(mut self, shader: Vec<u32>) -> Self {
    self.geo_shader = Some(shader);
    self
  }

  pub fn set_frag_shader(mut self, shader: Vec<u32>) -> Self {
    self.frag_shader = shader;
    self
  }

  pub fn add_descriptor_set(mut self, descriptor_set: DescriptorSet) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

pub struct ComputePipelineConfig {
  pub name: String,
  pub shader: Vec<u32>,
  pub descriptor_sets: Vec<DescriptorSet>,
}

impl ComputePipelineConfig {
  pub fn new(name: String) -> Self {
    Self {
      name,
      shader: Vec::new(),
      descriptor_sets: Vec::new(),
    }
  }

  pub fn set_shader(mut self, shader: Vec<u32>) -> Self {
    self.shader = shader;
    self
  }

  pub fn add_descriptor_set(mut self, descriptor_set: DescriptorSet) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

#[derive(Default, Clone)]
pub struct DescriptorSet {
  pub descriptors: Vec<Descriptor>,
}

pub enum DescriptorType {
  UniformBuffer,
  StorageBuffer,
}

impl DescriptorSet {
  pub fn add_descriptor(mut self, layout: Descriptor) -> Self {
    self.descriptors.push(layout);
    self
  }
}

#[derive(Clone)]
pub struct Descriptor {
  pub type_: vk::DescriptorType,
  pub buffer_usage: vk::BufferUsageFlags,
  pub descriptor_count: u32,
  pub stage: vk::ShaderStageFlags,
  pub size: u64,
}

impl Descriptor {
  pub fn new(
    type_: DescriptorType,
    descriptor_count: u32,
    stage: vk::ShaderStageFlags,
    size: u64,
  ) -> Self {
    let (type_, buffer_usage) = convert_type(type_);
    Self {
      type_,
      buffer_usage,
      descriptor_count,
      stage,
      size,
    }
  }
}

fn convert_type(type_: DescriptorType) -> (vk::DescriptorType, vk::BufferUsageFlags) {
  match type_ {
    DescriptorType::StorageBuffer => (
      vk::DescriptorType::STORAGE_BUFFER,
      vk::BufferUsageFlags::STORAGE_BUFFER,
    ),
    DescriptorType::UniformBuffer => (
      vk::DescriptorType::UNIFORM_BUFFER,
      vk::BufferUsageFlags::UNIFORM_BUFFER,
    ),
  }
}
