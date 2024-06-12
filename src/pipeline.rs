
use ash::vk;

use crate::swapchain::SwapchainDong;

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

pub struct Pipeline {
  pub pipeline: vk::Pipeline,
  pub pipeline_layout: vk::PipelineLayout,
  pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl Pipeline {
  pub fn init(
    logical_device: &ash::Device,
    swapchain_dong: &SwapchainDong,
    render_pass: vk::RenderPass,
  ) -> Result<Self, vk::Result> {
    let vertex_shader_create_info = vk::ShaderModuleCreateInfo::default()
      .code(vk_shader_macros::include_glsl!("./shaders/shader.vert"));
    let vertex_shader_module =
      unsafe { logical_device.create_shader_module(&vertex_shader_create_info, None) }?;

    let fragment_shader_create_info = vk::ShaderModuleCreateInfo::default()
      .code(vk_shader_macros::include_glsl!("./shaders/shader.frag"));
    let fragment_shader_module =
      unsafe { logical_device.create_shader_module(&fragment_shader_create_info, None) }?;

    let main_function_name = std::ffi::CString::new("main").unwrap();
    let vertex_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
      .stage(vk::ShaderStageFlags::VERTEX)
      .module(vertex_shader_module)
      .name(&main_function_name);
    let fragment_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
      .stage(vk::ShaderStageFlags::FRAGMENT)
      .module(fragment_shader_module)
      .name(&main_function_name);
    let shader_stages = [
      vertex_shader_stage_create_info,
      fragment_shader_stage_create_info,
    ];

    let vertex_attrib_descs = [
      vk::VertexInputAttributeDescription::default()
        .binding(0)
        .location(0)
        .offset(0)
        .format(vk::Format::R32G32B32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(0)
        .location(1)
        .offset(12)
        .format(vk::Format::R32G32B32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(2)
        .offset(0)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(3)
        .offset(16)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(4)
        .offset(32)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(5)
        .offset(48)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(6)
        .offset(64)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(7)
        .offset(80)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(8)
        .offset(96)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(9)
        .offset(112)
        .format(vk::Format::R32G32B32A32_SFLOAT),
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(10)
        .offset(128)
        .format(vk::Format::R32G32B32_SFLOAT),
    ];

    let vertex_binding_descs = [
      vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(24)
        .input_rate(vk::VertexInputRate::VERTEX),
      vk::VertexInputBindingDescription::default()
        .binding(1)
        .stride(140)
        .input_rate(vk::VertexInputRate::INSTANCE),
    ];

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
      .vertex_binding_descriptions(&vertex_binding_descs)
      .vertex_attribute_descriptions(&vertex_attrib_descs);
    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
      .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport = [vk::Viewport::default()
      .x(0.0)
      .y(0.0)
      .width(swapchain_dong.extent.width as f32)
      .height(swapchain_dong.extent.height as f32)
      .min_depth(0.0)
      .max_depth(1.0)];
    let scissor = [vk::Rect2D::default()
      .offset(vk::Offset2D::default())
      .extent(swapchain_dong.extent)];

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

    let descriptor_set_layout_binding_descs = [vk::DescriptorSetLayoutBinding::default()
      .binding(0)
      .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
      .descriptor_count(1)
      .stage_flags(vk::ShaderStageFlags::VERTEX)];
    let descriptor_set_layout_create_info =
      vk::DescriptorSetLayoutCreateInfo::default().bindings(&descriptor_set_layout_binding_descs);
    let descriptor_set_layout = unsafe {
      logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
    }?;
    let descriptor_set_layouts = vec![descriptor_set_layout];

    let pipeline_layout_create_info =
      vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_set_layouts);
    let pipeline_layout =
      unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;

    let depth_sencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
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
      .depth_stencil_state(&depth_sencil_info)
      .color_blend_state(&color_blend_info)
      .layout(pipeline_layout)
      .render_pass(render_pass)
      .subpass(0);

    let pipeline = unsafe {
      logical_device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        .expect("Unable to create graphics pipeline")
    }[0];

    unsafe {
      logical_device.destroy_shader_module(vertex_shader_module, None);
      logical_device.destroy_shader_module(fragment_shader_module, None);
    }

    Ok(Self {
      pipeline,
      pipeline_layout,
      descriptor_set_layouts,
    })
  }

  pub unsafe fn cleanup(&self, logical_device: &ash::Device) {
    for layout in &self.descriptor_set_layouts {
      logical_device.destroy_descriptor_set_layout(*layout, None);
    }
    logical_device.destroy_pipeline(self.pipeline, None);
    logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
  }
}