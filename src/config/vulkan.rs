use ash::vk;

pub use vk::ShaderStageFlags;

#[derive(Default)]
pub struct VulkanConfig {
  pub renderer: RendererConfig<'static>,
  pub shaders: Vec<PipelineType>,
  pub textures: Vec<&'static str>,
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

  pub fn add_texture(mut self, texture_path: &'static str) -> Self {
    self.textures.push(texture_path);
    self
  }

  pub fn add_textures(mut self, texture_paths: Vec<&'static str>) -> Self {
    self.textures.extend(texture_paths);
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
  pub descriptors: Vec<DescriptorType>,
}

#[derive(Clone)]
pub enum DescriptorType {
  UniformBuffer(BufferDescriptor),
  StorageBuffer(BufferDescriptor),
  Image(ImageDescriptor),
}

impl DescriptorType {
  pub fn new_storage(stage: vk::ShaderStageFlags, size: u64) -> Self {
    BufferDescriptor::new_storage(stage, size)
  }

  pub fn new_uniform(stage: vk::ShaderStageFlags, size: u64) -> Self {
    BufferDescriptor::new_uniform(stage, size)
  }

  pub fn new_image(stage: vk::ShaderStageFlags, paths: Vec<&str>) -> Self {
    ImageDescriptor::new_image(stage, paths)
  }
}

impl DescriptorSet {
  pub fn add_descriptor(mut self, layout: DescriptorType) -> Self {
    self.descriptors.push(layout);
    self
  }
}

#[derive(Clone)]
pub struct BufferDescriptor {
  pub type_: vk::DescriptorType,
  pub buffer_usage: vk::BufferUsageFlags,
  pub stage: vk::ShaderStageFlags,
  pub size: u64,
}

impl BufferDescriptor {
  pub fn new_storage(stage: vk::ShaderStageFlags, size: u64) -> DescriptorType {
    DescriptorType::StorageBuffer(Self {
      type_: vk::DescriptorType::STORAGE_BUFFER,
      buffer_usage: vk::BufferUsageFlags::STORAGE_BUFFER,
      stage,
      size,
    })
  }

  pub fn new_uniform(stage: vk::ShaderStageFlags, size: u64) -> DescriptorType {
    DescriptorType::UniformBuffer(Self {
      type_: vk::DescriptorType::UNIFORM_BUFFER,
      buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
      stage,
      size,
    })
  }
}

#[derive(Clone)]
pub struct ImageDescriptor {
  pub type_: vk::DescriptorType,
  pub stage: vk::ShaderStageFlags,
  pub paths: Vec<String>,
}

impl ImageDescriptor {
  pub fn new_image(stage: vk::ShaderStageFlags, paths: Vec<&str>) -> DescriptorType {
    DescriptorType::Image(Self {
      type_: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      stage,
      paths: paths.iter().map(|&s| s.to_string()).collect(),
    })
  }
}
