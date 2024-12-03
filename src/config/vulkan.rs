use ash::vk;

pub use vk::{Filter, ShaderStageFlags};

#[derive(Default)]
pub struct VulkanConfig {
  pub renderer: RendererConfig<'static>,
  pub shaders: Vec<PipelineType<'static>>,
  pub textures: Vec<ImageConfig<'static>>,
}

impl VulkanConfig {
  pub fn set_renderer_config(mut self, engine: RendererConfig<'static>) -> Self {
    self.renderer = engine;
    self
  }

  pub fn add_graphics_pipeline(mut self, pipeline: GraphicsPipelineConfig<'static>) -> Self {
    self.shaders.push(PipelineType::Graphics(pipeline));
    self
  }

  pub fn add_compute_pipeline(mut self, pipeline: ComputePipelineConfig<'static>) -> Self {
    self.shaders.push(PipelineType::Compute(pipeline));
    self
  }

  pub fn add_texture(mut self, texture_path: ImageConfig<'static>) -> Self {
    self.textures.push(texture_path);
    self
  }

  pub fn add_textures(mut self, texture_paths: Vec<ImageConfig<'static>>) -> Self {
    self.textures.extend(texture_paths);
    self
  }
}

#[derive(Clone)]
pub struct ImageConfig<'a> {
  pub interpolation: vk::Filter,
  pub data: ImageData<'a>,
}

#[derive(Clone)]
pub enum ImageData<'a> {
  Path(&'a str),
  Bytes(Vec<u8>),
}

impl<'a> ImageConfig<'a> {
  pub fn new_path(path: &'a str, interpolation: vk::Filter) -> ImageConfig<'a> {
    ImageConfig {
      interpolation,
      data: ImageData::Path(path),
    }
  }

  pub fn new_bytes(bytes: Vec<u8>, interpolation: vk::Filter) -> ImageConfig<'a> {
    ImageConfig {
      interpolation,
      data: ImageData::Bytes(bytes),
    }
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

pub enum PipelineType<'a> {
  Graphics(GraphicsPipelineConfig<'a>),
  Compute(ComputePipelineConfig<'a>),
}

pub struct GraphicsPipelineConfig<'a> {
  pub name: String,
  pub geo_shader: Option<Vec<u32>>,
  pub frag_shader: Vec<u32>,
  pub descriptor_sets: Vec<DescriptorSet<'a>>,
}

impl<'a> GraphicsPipelineConfig<'a> {
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

  pub fn add_descriptor_set(mut self, descriptor_set: DescriptorSet<'a>) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

pub struct ComputePipelineConfig<'a> {
  pub name: String,
  pub shader: Vec<u32>,
  pub descriptor_sets: Vec<DescriptorSet<'a>>,
}

impl<'a> ComputePipelineConfig<'a> {
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

  pub fn add_descriptor_set(mut self, descriptor_set: DescriptorSet<'a>) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

#[derive(Default, Clone)]
pub struct DescriptorSet<'a> {
  pub descriptors: Vec<DescriptorType<'a>>,
}

#[derive(Clone)]
pub enum DescriptorType<'a> {
  UniformBuffer(BufferDescriptor),
  StorageBuffer(BufferDescriptor),
  Image(ImageDescriptor<'a>),
}

impl<'a> DescriptorType<'a> {
  pub fn new_storage(stage: vk::ShaderStageFlags, size: usize) -> Self {
    BufferDescriptor::new_storage(stage, size)
  }

  pub fn new_uniform(stage: vk::ShaderStageFlags, size: usize) -> Self {
    BufferDescriptor::new_uniform(stage, size)
  }

  pub fn new_image(stage: vk::ShaderStageFlags, images: Vec<ImageConfig<'a>>) -> Self {
    ImageDescriptor::new_image(stage, images)
  }
}

impl<'a> DescriptorSet<'a> {
  pub fn add_descriptor(mut self, layout: DescriptorType<'a>) -> Self {
    self.descriptors.push(layout);
    self
  }
}

#[derive(Clone)]
pub struct BufferDescriptor {
  pub type_: vk::DescriptorType,
  pub buffer_usage: vk::BufferUsageFlags,
  pub stage: vk::ShaderStageFlags,
  pub size: usize,
}

impl BufferDescriptor {
  pub fn new_storage(stage: vk::ShaderStageFlags, size: usize) -> DescriptorType<'static> {
    DescriptorType::StorageBuffer(Self {
      type_: vk::DescriptorType::STORAGE_BUFFER,
      buffer_usage: vk::BufferUsageFlags::STORAGE_BUFFER,
      stage,
      size,
    })
  }

  pub fn new_uniform(stage: vk::ShaderStageFlags, size: usize) -> DescriptorType<'static> {
    DescriptorType::UniformBuffer(Self {
      type_: vk::DescriptorType::UNIFORM_BUFFER,
      buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
      stage,
      size,
    })
  }
}

#[derive(Clone)]
pub struct ImageDescriptor<'a> {
  pub type_: vk::DescriptorType,
  pub stage: vk::ShaderStageFlags,
  pub images: Vec<ImageConfig<'a>>,
}

impl<'a> ImageDescriptor<'a> {
  pub fn new_image(stage: vk::ShaderStageFlags, images: Vec<ImageConfig<'a>>) -> DescriptorType<'a> {
    DescriptorType::Image(Self {
      type_: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      stage,
      images,
    })
  }
}
