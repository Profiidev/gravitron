use ash::vk;

use crate::config::vulkan::{
  ComputePipelineConfig, Descriptor, DescriptorSet, GraphicsPipelineConfig, PipelineType,
  ShaderConfig, ShaderInputBindings, ShaderInputVariable, ShaderType,
};

pub fn init_render_pass(
  logical_device: &ash::Device,
  format: vk::Format,
) -> Result<vk::RenderPass, vk::Result> {
  let attachment = [
    vk::AttachmentDescription::default()
      .format(format)
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::STORE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
    vk::AttachmentDescription::default()
      .format(vk::Format::D32_SFLOAT)
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::DONT_CARE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
  ];

  let color_attachment_ref = [vk::AttachmentReference::default()
    .attachment(0)
    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
  let depth_attachment_ref = vk::AttachmentReference::default()
    .attachment(1)
    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

  let subpass = [vk::SubpassDescription::default()
    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    .depth_stencil_attachment(&depth_attachment_ref)
    .color_attachments(&color_attachment_ref)];

  let subpass_dependency = [vk::SubpassDependency::default()
    .src_subpass(vk::SUBPASS_EXTERNAL)
    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    .dst_subpass(0)
    .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    .dst_access_mask(
      vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
    )];

  let render_pass_create_info = vk::RenderPassCreateInfo::default()
    .attachments(&attachment)
    .subpasses(&subpass)
    .dependencies(&subpass_dependency);
  unsafe { logical_device.create_render_pass(&render_pass_create_info, None) }
}

pub struct PipelineManager {
  pub pipelines: Vec<Pipeline>,
}

impl PipelineManager {
  pub fn init(
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
    swap_chain_extent: &vk::Extent2D,
    pipelines: &mut Vec<PipelineType>,
  ) -> Result<Self, vk::Result> {
    pipelines.push(PipelineType::Graphics(Pipeline::default_shader(
      swap_chain_extent,
    )));

    let mut vk_pipelines = vec![];
    for pipeline in pipelines {
      match pipeline {
        PipelineType::Graphics(config) => {
          vk_pipelines.push(Pipeline::init_graphics_pipeline(
            logical_device,
            render_pass,
            config,
          )?);
        }
        PipelineType::Compute(config) => {
          vk_pipelines.push(Pipeline::init_compute_pipeline(logical_device, config)?);
        }
      }
    }

    Ok(Self {
      pipelines: vk_pipelines,
    })
  }

  pub fn destroy(&self, logical_device: &ash::Device) {
    std::fs::create_dir_all("cache").unwrap();
    for pipeline in &self.pipelines {
      pipeline.destroy(logical_device);
    }
  }
}

pub struct Pipeline {
  name: String,
  pub pipeline: vk::Pipeline,
  pub pipeline_layout: vk::PipelineLayout,
  pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
  cache: vk::PipelineCache,
}

impl Pipeline {
  pub fn default_shader(extend: &vk::Extent2D) -> GraphicsPipelineConfig {
    GraphicsPipelineConfig::new(
      "default".to_string(),
      vk::PrimitiveTopology::TRIANGLE_LIST,
      (extend.width, extend.height),
    )
    .add_shader(ShaderConfig::new(
      ShaderType::Vertex,
      vk_shader_macros::include_glsl!("./shaders/shader.vert").to_vec(),
    ))
    .add_shader(ShaderConfig::new(
      ShaderType::Fragment,
      vk_shader_macros::include_glsl!("./shaders/shader.frag").to_vec(),
    ))
    .add_input(
      ShaderInputBindings::new(vk::VertexInputRate::VERTEX)
        .add_variable(ShaderInputVariable::Vec3)
        .add_variable(ShaderInputVariable::Vec3),
    )
    .add_input(
      ShaderInputBindings::new(vk::VertexInputRate::INSTANCE)
        .add_variable(ShaderInputVariable::Mat4)
        .add_variable(ShaderInputVariable::Mat4)
        .add_variable(ShaderInputVariable::Vec3)
        .add_variable(ShaderInputVariable::Float)
        .add_variable(ShaderInputVariable::Float),
    )
    .add_descriptor_set(DescriptorSet::default().add_descriptor(Descriptor::new(
      vk::DescriptorType::UNIFORM_BUFFER,
      1,
      vk::ShaderStageFlags::VERTEX,
    )))
    .add_descriptor_set(DescriptorSet::default().add_descriptor(Descriptor::new(
      vk::DescriptorType::STORAGE_BUFFER,
      1,
      vk::ShaderStageFlags::FRAGMENT,
    )))
  }

  pub fn init_compute_pipeline(
    logical_device: &ash::Device,
    pipeline: &ComputePipelineConfig,
  ) -> Result<Self, vk::Result> {
    let main_function_name = std::ffi::CString::new("main").unwrap();

    let shader_create_info = vk::ShaderModuleCreateInfo::default().code(&pipeline.shader.code);
    let shader_module = unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;

    let shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
      .stage(pipeline.shader.type_)
      .module(shader_module)
      .name(&main_function_name);

    let descriptor_layouts =
      Self::get_descriptor_set_layouts(&pipeline.descriptor_sets, logical_device)?;

    let pipeline_layout_create_info =
      vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_layouts);
    let pipeline_layout =
      unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;

    let pipeline_create_info = vk::ComputePipelineCreateInfo::default()
      .stage(shader_stage_create_info)
      .layout(pipeline_layout);

    let pipeline_cache = Self::create_shader_cache(logical_device, &pipeline.name)?;

    let vk_pipelines = unsafe {
      logical_device
        .create_compute_pipelines(pipeline_cache, &[pipeline_create_info], None)
        .expect("Unable to create compute pipeline")
    }[0];

    unsafe {
      logical_device.destroy_shader_module(shader_module, None);
    }

    Ok(Self {
      name: pipeline.name.clone(),
      pipeline: vk_pipelines,
      pipeline_layout,
      descriptor_set_layouts: descriptor_layouts,
      cache: pipeline_cache,
    })
  }

  pub fn init_graphics_pipeline(
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
    pipeline: &GraphicsPipelineConfig,
  ) -> Result<Self, vk::Result> {
    let main_function_name = std::ffi::CString::new("main").unwrap();

    let mut shader_modules = vec![];
    for shader in &pipeline.shaders {
      let shader_create_info = vk::ShaderModuleCreateInfo::default().code(&shader.code);
      let shader_module =
        unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
      shader_modules.push((shader_module, shader.type_));
    }

    let mut shader_stages = vec![];
    for shader in &shader_modules {
      let shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
        .stage(shader.1)
        .module(shader.0)
        .name(&main_function_name);
      shader_stages.push(shader_stage_create_info);
    }

    let mut vertex_attrib_descs = vec![];
    let mut vertex_binding_descs = vec![];

    for (i, input) in pipeline.input.iter().enumerate() {
      let mut current_offset = 0;

      for variable in &input.variables {
        let mut times_to_add = 1;
        let mut size = 4;

        let format = match variable {
          ShaderInputVariable::Float => vk::Format::R32_SFLOAT,
          ShaderInputVariable::Vec2 => {
            size = 8;
            vk::Format::R32G32_SFLOAT
          }
          ShaderInputVariable::Vec3 => {
            size = 12;
            vk::Format::R32G32B32_SFLOAT
          }
          ShaderInputVariable::Vec4 => {
            size = 16;
            vk::Format::R32G32B32A32_SFLOAT
          }
          ShaderInputVariable::Mat2 => {
            size = 8;
            times_to_add = 2;
            vk::Format::R32G32_SFLOAT
          }
          ShaderInputVariable::Mat3 => {
            size = 12;
            times_to_add = 3;
            vk::Format::R32G32B32_SFLOAT
          }
          ShaderInputVariable::Mat4 => {
            size = 16;
            times_to_add = 4;
            vk::Format::R32G32B32A32_SFLOAT
          }
          ShaderInputVariable::Int => vk::Format::R32_SINT,
          ShaderInputVariable::UInt => vk::Format::R32_UINT,
          ShaderInputVariable::Double => {
            size = 8;
            vk::Format::R64_SFLOAT
          }
        };

        for _ in 0..times_to_add {
          vertex_attrib_descs.push(
            vk::VertexInputAttributeDescription::default()
              .binding(i as u32)
              .location(vertex_attrib_descs.len() as u32)
              .offset(current_offset)
              .format(format),
          );
          current_offset += size;
        }
      }

      vertex_binding_descs.push(
        vk::VertexInputBindingDescription::default()
          .binding(i as u32)
          .stride(current_offset)
          .input_rate(input.input_rate),
      );
    }

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
      .vertex_binding_descriptions(&vertex_binding_descs)
      .vertex_attribute_descriptions(&vertex_attrib_descs);
    let input_assembly_info =
      vk::PipelineInputAssemblyStateCreateInfo::default().topology(pipeline.topology);

    let viewport = [vk::Viewport::default()
      .x(0.0)
      .y(0.0)
      .width(pipeline.viewport_size.0 as f32)
      .height(pipeline.viewport_size.1 as f32)
      .min_depth(0.0)
      .max_depth(1.0)];
    let scissor = [vk::Rect2D::default()
      .offset(vk::Offset2D::default())
      .extent(vk::Extent2D {
        width: pipeline.viewport_size.0,
        height: pipeline.viewport_size.1,
      })];

    let viewport_info = vk::PipelineViewportStateCreateInfo::default()
      .viewports(&viewport)
      .scissors(&scissor);

    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
      .line_width(1.0)
      .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
      .cull_mode(vk::CullModeFlags::BACK)
      .polygon_mode(vk::PolygonMode::FILL);

    let multisample_info = vk::PipelineMultisampleStateCreateInfo::default()
      .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let color_blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
      .color_write_mask(
        vk::ColorComponentFlags::R
          | vk::ColorComponentFlags::G
          | vk::ColorComponentFlags::B
          | vk::ColorComponentFlags::A,
      )
      .blend_enable(false)
      .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .color_blend_op(vk::BlendOp::ADD)
      .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .alpha_blend_op(vk::BlendOp::ADD)];
    let color_blend_info =
      vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachment);

    let descriptor_layouts =
      Self::get_descriptor_set_layouts(&pipeline.descriptor_sets, logical_device)?;

    let pipeline_layout_create_info =
      vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_layouts);
    let pipeline_layout =
      unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;

    let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
      .depth_test_enable(true)
      .depth_write_enable(true)
      .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
      .stages(&shader_stages)
      .vertex_input_state(&vertex_input_info)
      .input_assembly_state(&input_assembly_info)
      .viewport_state(&viewport_info)
      .rasterization_state(&rasterizer_info)
      .multisample_state(&multisample_info)
      .depth_stencil_state(&depth_stencil_info)
      .color_blend_state(&color_blend_info)
      .layout(pipeline_layout)
      .render_pass(render_pass)
      .subpass(0);

    let pipeline_cache = Self::create_shader_cache(logical_device, &pipeline.name)?;

    let vk_pipelines = unsafe {
      logical_device
        .create_graphics_pipelines(pipeline_cache, &[pipeline_create_info], None)
        .expect("Unable to create graphics pipeline")
    }[0];

    for module in shader_modules {
      unsafe {
        logical_device.destroy_shader_module(module.0, None);
      }
    }

    Ok(Self {
      name: pipeline.name.clone(),
      pipeline: vk_pipelines,
      pipeline_layout,
      descriptor_set_layouts: descriptor_layouts,
      cache: pipeline_cache,
    })
  }

  fn get_descriptor_set_layouts(
    descriptor_sets: &Vec<DescriptorSet>,
    logical_device: &ash::Device,
  ) -> Result<Vec<vk::DescriptorSetLayout>, vk::Result> {
    let mut descriptor_layouts = vec![];
    for descriptor_set in descriptor_sets {
      let mut descriptor_set_layout_binding_descs = vec![];
      for (i, descriptor) in descriptor_set.descriptors.iter().enumerate() {
        descriptor_set_layout_binding_descs.push(
          vk::DescriptorSetLayoutBinding::default()
            .binding(i as u32)
            .descriptor_type(descriptor.type_)
            .descriptor_count(1)
            .stage_flags(descriptor.stage),
        );
      }

      let descriptor_set_layout_create_info =
        vk::DescriptorSetLayoutCreateInfo::default().bindings(&descriptor_set_layout_binding_descs);
      let descriptor_set_layout = unsafe {
        logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
      }?;
      descriptor_layouts.push(descriptor_set_layout);
    }

    Ok(descriptor_layouts)
  }

  fn create_shader_cache(
    logical_device: &ash::Device,
    name: &str,
  ) -> Result<vk::PipelineCache, vk::Result> {
    let initial_data = std::fs::read(format!("cache/{}.bin", name)).unwrap_or_default();

    let pipeline_cache_create_info =
      vk::PipelineCacheCreateInfo::default().initial_data(&initial_data);

    unsafe { logical_device.create_pipeline_cache(&pipeline_cache_create_info, None) }
  }

  pub fn destroy(&self, logical_device: &ash::Device) {
    unsafe {
      for layout in &self.descriptor_set_layouts {
        logical_device.destroy_descriptor_set_layout(*layout, None);
      }
      logical_device.destroy_pipeline(self.pipeline, None);
      logical_device.destroy_pipeline_layout(self.pipeline_layout, None);

      let pipeline_cache_data = logical_device.get_pipeline_cache_data(self.cache).unwrap();
      std::fs::write(format!("cache/{}.bin", self.name), pipeline_cache_data).unwrap();
      logical_device.destroy_pipeline_cache(self.cache, None);
    }
  }
}
