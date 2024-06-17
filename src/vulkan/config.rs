use ash::vk;

use crate::utils::LogLevel;

#[derive(Default)]
pub(crate) struct VulkanConfig {
  pub(super) engine: EngineConfig<'static>,
  pub(super) app: AppConfig,
  pub(super) shaders: Vec<PipelineType>,
}

impl VulkanConfig {
  pub(crate) fn set_engine_config(mut self, engine: EngineConfig<'static>) -> Self {
    self.engine = engine;
    self
  }

  pub(crate) fn set_app_config(mut self, app: AppConfig) -> Self {
    self.app = app;
    self
  }

  pub(crate) fn add_graphics_pipeline(mut self, pipeline: GraphicsPipelineConfig) -> Self {
    self.shaders.push(PipelineType::Graphics(pipeline));
    self
  }

  pub(crate) fn add_compute_pipeline(mut self, pipeline: ComputePipelineConfig) -> Self {
    self.shaders.push(PipelineType::Compute(pipeline));
    self
  }
}

#[derive(Default)]
pub(crate) struct EngineConfig<'a> {
  pub(super) layers: Vec<&'a std::ffi::CStr>,
  pub(super) instance_extensions: Vec<&'a std::ffi::CStr>,
  pub(super) instance_next: Vec<Box<dyn vk::ExtendsInstanceCreateInfo>>,
  pub(super) device_extensions: Vec<&'a std::ffi::CStr>,
  pub(super) device_features: vk::PhysicalDeviceFeatures,
  pub(super) debug: bool,
  pub(super) debug_log_level: vk::DebugUtilsMessageSeverityFlagsEXT,
}

impl<'a> EngineConfig<'a> {
  pub(crate) fn add_layer(mut self, layer: &'a std::ffi::CStr) -> Self {
    self.layers.push(layer);
    self
  }

  pub(crate) fn set_debug(mut self, debug: bool) -> Self {
    self.debug = debug;
    self
  }

  pub(crate) fn set_debug_log_level(mut self, level: LogLevel) -> Self {
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

pub(crate) struct AppConfig {
  pub(super) title: String,
  pub(super) version: u32,
  pub(super) width: u32,
  pub(super) height: u32,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      title: "Vulkan Test App".to_string(),
      version: vk::make_api_version(0, 0, 1, 0),
      width: 800,
      height: 600,
    }
  }
}

pub(crate) enum PipelineType {
  Graphics(GraphicsPipelineConfig),
  Compute(ComputePipelineConfig),
}

pub(crate) struct GraphicsPipelineConfig {
  pub(super) name: String,
  pub(super) shaders: Vec<ShaderConfig>,
  pub(super) input: Vec<ShaderInputBindings>,
  pub(super) topology: vk::PrimitiveTopology,
  pub(super) viewport_size: (u32, u32),
  pub(super) descriptor_sets: Vec<DescriptorSet>,
}

impl GraphicsPipelineConfig {
  pub(crate) fn new(
    name: String,
    topology: vk::PrimitiveTopology,
    viewport_size: (u32, u32),
  ) -> Self {
    Self {
      name,
      shaders: Vec::new(),
      input: Vec::new(),
      topology,
      viewport_size,
      descriptor_sets: Vec::new(),
    }
  }

  pub(crate) fn add_shader(mut self, shader: ShaderConfig) -> Self {
    self.shaders.push(shader);
    self
  }

  pub(crate) fn add_input(mut self, input: ShaderInputBindings) -> Self {
    self.input.push(input);
    self
  }

  pub(crate) fn add_descriptor_set(mut self, descriptor_set: DescriptorSet) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

pub(crate) struct ComputePipelineConfig {
  pub(super) name: String,
  pub(super) shader: ShaderConfig,
  pub(super) descriptor_sets: Vec<DescriptorSet>,
}

impl ComputePipelineConfig {
  pub(crate) fn new(name: String) -> Self {
    Self {
      name,
      shader: ShaderConfig {
        type_: vk::ShaderStageFlags::COMPUTE,
        code: Vec::new(),
      },
      descriptor_sets: Vec::new(),
    }
  }

  pub(crate) fn set_shader(mut self, shader: ShaderConfig) -> Self {
    self.shader = shader;
    self
  }

  pub(crate) fn add_descriptor_set(mut self, descriptor_set: DescriptorSet) -> Self {
    self.descriptor_sets.push(descriptor_set);
    self
  }
}

pub(crate) struct ShaderConfig {
  pub(super) type_: vk::ShaderStageFlags,
  pub(super) code: Vec<u32>,
}

impl ShaderConfig {
  pub(crate) fn new(type_: ShaderType, code: Vec<u32>) -> Self {
    let type_ = match type_ {
      ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
      ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
      ShaderType::Compute => vk::ShaderStageFlags::COMPUTE,
      ShaderType::Geometry => vk::ShaderStageFlags::GEOMETRY,
    };
    Self { type_, code }
  }
}

pub(crate) enum ShaderType {
  Vertex,
  Fragment,
  Compute,
  Geometry,
}

pub(crate) struct ShaderInputBindings {
  pub(super) input_rate: vk::VertexInputRate,
  pub(super) variables: Vec<ShaderInputVariable>,
}

impl ShaderInputBindings {
  pub(crate) fn new(input_rate: vk::VertexInputRate) -> Self {
    Self {
      input_rate,
      variables: Vec::new(),
    }
  }

  pub(crate) fn add_variable(mut self, variable: ShaderInputVariable) -> Self {
    self.variables.push(variable);
    self
  }
}

pub(crate) enum ShaderInputVariable {
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

pub(crate) struct DescriptorSet {
  pub(super) descriptors: Vec<Descriptor>,
}

impl DescriptorSet {
  pub(crate) fn new() -> Self {
    Self {
      descriptors: Vec::new(),
    }
  }

  pub(crate) fn add_descriptor(mut self, layout: Descriptor) -> Self {
    self.descriptors.push(layout);
    self
  }
}

pub(crate) struct Descriptor {
  pub(super) type_: vk::DescriptorType,
  pub(super) descriptor_count: u32,
  pub(super) stage: vk::ShaderStageFlags,
}

impl Descriptor {
  pub(crate) fn new(
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
