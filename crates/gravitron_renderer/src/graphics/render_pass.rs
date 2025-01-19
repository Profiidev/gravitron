use ash::vk;

pub fn init_render_pass(
  logical_device: &ash::Device,
  pipeline_count: usize,
) -> Result<vk::RenderPass, vk::Result> {
  let color = vk::AttachmentDescription::default()
    .format(vk::Format::R32G32B32A32_SFLOAT)
    .samples(vk::SampleCountFlags::TYPE_1)
    .load_op(vk::AttachmentLoadOp::CLEAR)
    .store_op(vk::AttachmentStoreOp::STORE)
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
    .initial_layout(vk::ImageLayout::UNDEFINED)
    .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
  let normal = color;
  let pos = color;

  let depth = vk::AttachmentDescription::default()
    .format(vk::Format::D32_SFLOAT)
    .samples(vk::SampleCountFlags::TYPE_1)
    .load_op(vk::AttachmentLoadOp::CLEAR)
    .store_op(vk::AttachmentStoreOp::STORE)
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
    .initial_layout(vk::ImageLayout::UNDEFINED)
    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

  let attachment = [color, normal, pos, depth];

  let color = [
    vk::AttachmentReference::default()
      .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
      .attachment(0),
    vk::AttachmentReference::default()
      .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
      .attachment(1),
    vk::AttachmentReference::default()
      .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
      .attachment(2),
  ];
  let depth = vk::AttachmentReference::default()
    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
    .attachment(3);

  let mut subpass = Vec::new();
  for _ in 0..pipeline_count {
    subpass.push(
      vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .depth_stencil_attachment(&depth)
        .color_attachments(&color),
    );
  }

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

pub fn init_light_render_pass(
  logical_device: &ash::Device,
  format: vk::Format,
) -> Result<vk::RenderPass, vk::Result> {
  let color = vk::AttachmentDescription::default()
    .format(format)
    .samples(vk::SampleCountFlags::TYPE_1)
    .load_op(vk::AttachmentLoadOp::CLEAR)
    .store_op(vk::AttachmentStoreOp::STORE)
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
    .initial_layout(vk::ImageLayout::UNDEFINED)
    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

  let attachment = [color];

  let color = [vk::AttachmentReference::default()
    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    .attachment(0)];

  let subpass = [vk::SubpassDescription::default()
    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    .color_attachments(&color)];

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
