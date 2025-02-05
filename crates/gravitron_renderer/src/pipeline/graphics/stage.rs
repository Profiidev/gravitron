use ash::vk;
use RenderingStage::*;

#[derive(Default, PartialEq, PartialOrd)]
pub enum RenderingStage {
  Light,
  #[default]
  World,
}

impl RenderingStage {
  #[inline]
  pub(crate) fn vertex_shader(&self) -> &'static [u32] {
    match self {
      Light => vk_shader_macros::include_glsl!("./assets/light.vert"),
      World => vk_shader_macros::include_glsl!("./assets/shader.vert"),
    }
  }

  #[inline]
  pub(crate) fn fragment_shader(&self) -> &'static [u32] {
    match self {
      Light => vk_shader_macros::include_glsl!("./assets/light.frag"),
      World => vk_shader_macros::include_glsl!("./assets/shader.frag"),
    }
  }

  #[inline]
  pub(crate) fn inputs(
    &self,
  ) -> (
    Vec<vk::VertexInputBindingDescription>,
    Vec<vk::VertexInputAttributeDescription>,
  ) {
    match self {
      Light => (vec![], vec![]),
      World => {
        let vertex_binding = vec![
          vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(32)
            .input_rate(vk::VertexInputRate::VERTEX),
          vk::VertexInputBindingDescription::default()
            .binding(1)
            .stride(156)
            .input_rate(vk::VertexInputRate::INSTANCE),
        ];

        let mut vertex_attrib = vec![];

        for i in 0..2 {
          vertex_attrib.push(
            vk::VertexInputAttributeDescription::default()
              .binding(0)
              .location(i)
              .offset(i * 12)
              .format(vk::Format::R32G32B32_SFLOAT),
          );
        }

        vertex_attrib.push(
          vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(2)
            .offset(24)
            .format(vk::Format::R32G32_SFLOAT),
        );

        for i in 0..9 {
          vertex_attrib.push(
            vk::VertexInputAttributeDescription::default()
              .binding(1)
              .location(i + 3)
              .offset(i * 16)
              .format(vk::Format::R32G32B32A32_SFLOAT),
          );
        }

        for i in 0..2 {
          vertex_attrib.push(
            vk::VertexInputAttributeDescription::default()
              .binding(1)
              .location(12 + i)
              .offset(144 + i * 4)
              .format(vk::Format::R32_SFLOAT),
          );
        }

        vertex_attrib.push(
          vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(14)
            .offset(152)
            .format(vk::Format::R32_UINT),
        );

        (vertex_binding, vertex_attrib)
      }
    }
  }

  #[inline]
  pub(crate) fn output(
    &self,
    color: vk::PipelineColorBlendAttachmentState,
  ) -> Vec<vk::PipelineColorBlendAttachmentState> {
    match self {
      Light => vec![color],
      World => vec![color, color, color],
    }
  }

  #[inline]
  pub(crate) fn depth_buffer<'d>(
    &self,
    info: vk::GraphicsPipelineCreateInfo<'d>,
    depth: &'d vk::PipelineDepthStencilStateCreateInfo,
  ) -> vk::GraphicsPipelineCreateInfo<'d> {
    match self {
      World => info.depth_stencil_state(depth),
      Light => info,
    }
  }

  #[inline]
  pub(crate) fn subpass(&self) -> u32 {
    match self {
      Light => 1,
      World => 0,
    }
  }

  #[inline]
  pub(crate) fn cull_mode(&self) -> vk::CullModeFlags {
    match self {
      Light => vk::CullModeFlags::NONE,
      World => vk::CullModeFlags::BACK,
    }
  }
}
