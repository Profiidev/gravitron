use ash::vk;

use super::utils::LogLevel;

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
  pub debug: bool,
  pub debug_log_level: vk::DebugUtilsMessageSeverityFlagsEXT,
}

impl<'a> RendererConfig<'a> {
  pub fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }

  pub fn set_debug(mut self, debug: bool) -> Self {
    self.debug = debug;
    self
  }

  pub fn set_debug_log_level(mut self, level: LogLevel) -> Self {
    self.debug_log_level = match level {
      LogLevel::Info => {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO
          | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
          | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      }
      LogLevel::Verbose => {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
          | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      }
      LogLevel::Warning => {
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
      }
      LogLevel::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
      LogLevel::None => vk::DebugUtilsMessageSeverityFlagsEXT::empty(),
    };
    self
  }
}

pub enum PipelineType {
  Graphics(GraphicsPipelineConfig),
  Compute(ComputePipelineConfig),
}

pub struct GraphicsPipelineConfig {
  pub name: String,
  pub shaders: Vec<ShaderConfig>,
  pub input: Vec<ShaderInputBindings>,
  pub topology: vk::PrimitiveTopology,
  pub viewport_size: (u32, u32),
  pub descriptor_sets: Vec<DescriptorSet>,
}

impl GraphicsPipelineConfig {
  pub fn new(name: String, topology: vk::PrimitiveTopology, viewport_size: (u32, u32)) -> Self {
    Self {
      name,
      shaders: Vec::new(),
      input: Vec::new(),
      topology,
      viewport_size,
      descriptor_sets: Vec::new(),
    }
  }

  pub fn add_shader(mut self, shader: ShaderConfig) -> Self {
    self.shaders.push(shader);
    self
  }

  pub fn add_input(mut self, input: ShaderInputBindings) -> Self {
    self.input.push(input);
    self
  }

  pub fn add_descriptor_set(mut self, descriptor_set: DescriptorSet) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

pub struct ComputePipelineConfig {
  pub name: String,
  pub shader: ShaderConfig,
  pub descriptor_sets: Vec<DescriptorSet>,
}

impl ComputePipelineConfig {
  pub fn new(name: String) -> Self {
    Self {
      name,
      shader: ShaderConfig {
        type_: vk::ShaderStageFlags::COMPUTE,
        code: Vec::new(),
      },
      descriptor_sets: Vec::new(),
    }
  }

  pub fn set_shader(mut self, shader: ShaderConfig) -> Self {
    self.shader = shader;
    self
  }

  pub fn add_descriptor_set(mut self, descriptor_set: DescriptorSet) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

pub struct ShaderConfig {
  pub type_: vk::ShaderStageFlags,
  pub code: Vec<u32>,
}

impl ShaderConfig {
  pub fn new(type_: ShaderType, code: Vec<u32>) -> Self {
    let type_ = match type_ {
      ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
      ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
      ShaderType::Compute => vk::ShaderStageFlags::COMPUTE,
      ShaderType::Geometry => vk::ShaderStageFlags::GEOMETRY,
    };
    Self { type_, code }
  }
}

pub enum ShaderType {
  Vertex,
  Fragment,
  Compute,
  Geometry,
}

pub struct ShaderInputBindings {
  pub input_rate: vk::VertexInputRate,
  pub variables: Vec<ShaderInputVariable>,
}

impl ShaderInputBindings {
  pub fn new(input_rate: vk::VertexInputRate) -> Self {
    Self {
      input_rate,
      variables: Vec::new(),
    }
  }

  pub fn add_variable(mut self, variable: ShaderInputVariable) -> Self {
    self.variables.push(variable);
    self
  }
}

pub enum ShaderInputVariable {
  Float,
  Vec2,
  Vec3,
  Vec4,
  Mat2,
  Mat3,
  Mat4,
  Int,
  UInt,
  Double,
}

#[derive(Default)]
pub struct DescriptorSet {
  pub descriptors: Vec<Descriptor>,
}

impl DescriptorSet {
  pub fn add_descriptor(mut self, layout: Descriptor) -> Self {
    self.descriptors.push(layout);
    self
  }
}

pub struct Descriptor {
  pub type_: vk::DescriptorType,
  pub descriptor_count: u32,
  pub stage: vk::ShaderStageFlags,
}

impl Descriptor {
  pub fn new(
    type_: vk::DescriptorType,
    descriptor_count: u32,
    stage: vk::ShaderStageFlags,
  ) -> Self {
    Self {
      type_,
      descriptor_count,
      stage,
    }
  }
}
