use ash::vk;

pub fn init_render_pass(
  logical_device: &ash::Device,
  format: vk::Format,
  pipeline_count: usize,
) -> Result<vk::RenderPass, vk::Result> {
  let color = vk::AttachmentDescription::default()
    .format(format)
    .samples(vk::SampleCountFlags::TYPE_1)
    .load_op(vk::AttachmentLoadOp::LOAD)
    .store_op(vk::AttachmentStoreOp::STORE)
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
    .initial_layout(vk::ImageLayout::PRESENT_SRC_KHR)
    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
  let depth = vk::AttachmentDescription::default()
    .format(vk::Format::D32_SFLOAT)
    .samples(vk::SampleCountFlags::TYPE_1)
    .load_op(vk::AttachmentLoadOp::LOAD)
    .store_op(vk::AttachmentStoreOp::STORE)
    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

  let attachment = if pipeline_count == 1 {
    vec![
      color
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .initial_layout(vk::ImageLayout::UNDEFINED),
      depth
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .initial_layout(vk::ImageLayout::UNDEFINED),
    ]
  } else if pipeline_count == 2 {
    vec![
      color
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .initial_layout(vk::ImageLayout::UNDEFINED),
      depth
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .initial_layout(vk::ImageLayout::UNDEFINED),
      color,
      depth,
    ]
  } else {
    vec![
      color
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .initial_layout(vk::ImageLayout::UNDEFINED),
      depth
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .initial_layout(vk::ImageLayout::UNDEFINED),
      color,
      depth,
      color,
      depth,
    ]
  };

  let end = if pipeline_count <= 2 { 2 } else { 4 };

  let color = vk::AttachmentReference::default().layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
  let depth =
    vk::AttachmentReference::default().layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

  let start_color_ref = [color.attachment(0)];
  let middle_color_ref = [color.attachment(2)];
  let end_color_ref = [color.attachment(end)];

  let start_depth_ref = depth.attachment(1);
  let middle_depth_ref = depth.attachment(3);
  let end_depth_ref = depth.attachment(end + 1);

  let mut subpass = Vec::new();
  for i in 0..pipeline_count {
    if i == 0 {
      subpass.push(
        vk::SubpassDescription::default()
          .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
          .depth_stencil_attachment(&start_depth_ref)
          .color_attachments(&start_color_ref),
      );
    } else if pipeline_count - 1 == i {
      subpass.push(
        vk::SubpassDescription::default()
          .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
          .depth_stencil_attachment(&end_depth_ref)
          .color_attachments(&end_color_ref),
      );
    } else {
      subpass.push(
        vk::SubpassDescription::default()
          .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
          .depth_stencil_attachment(&middle_depth_ref)
          .color_attachments(&middle_color_ref),
      );
    }
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