use std::collections::HashMap;

use anyhow::Error;
use ash::vk;

use crate::{
  config::vulkan::{
    ComputePipelineConfig, DescriptorSet, DescriptorType, GraphicsPipelineConfig, ImageConfig,
    PipelineType,
  },
  ecs::components::camera::Camera,
};

use super::memory::{
  manager::MemoryManager,
  types::{BufferBlockSize, BufferId},
  BufferMemory,
};

pub mod pools;

pub struct PipelineManager {
  pipelines: Vec<(String, Pipeline)>,
  descriptor_pool: vk::DescriptorPool,
  logical_device: ash::Device,
  default_desc_layouts: Vec<vk::DescriptorSetLayout>,
  default_buffers: HashMap<usize, HashMap<usize, BufferId>>,
}

impl PipelineManager {
  pub fn init(
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
    swap_chain_extent: &vk::Extent2D,
    pipelines: &mut Vec<PipelineType>,
    textures: Vec<ImageConfig>,
    memory_manager: &mut MemoryManager,
  ) -> Result<Self, Error> {
    pipelines.push(PipelineType::Graphics(Pipeline::default_shader()));

    let mut descriptor_count = 1;
    let mut pool_sizes = vec![];

    let mut textures_used = vec![ImageConfig::Bytes(
      include_bytes!("../../../assets/default.png").to_vec(),
    )];
    textures_used.extend(textures);

    let default_descriptor_set = DescriptorSet::default()
      .add_descriptor(DescriptorType::new_uniform(
        vk::ShaderStageFlags::VERTEX,
        128,
      ))
      .add_descriptor(DescriptorType::new_image(
        vk::ShaderStageFlags::FRAGMENT,
        textures_used,
      ));
    for desc in &default_descriptor_set.descriptors {
      add_descriptor(&mut pool_sizes, desc);
    }

    for pipeline in &*pipelines {
      match pipeline {
        PipelineType::Graphics(c) => {
          descriptor_count += c.descriptor_sets.len();
          for descriptor_set in &c.descriptor_sets {
            for descriptor in &descriptor_set.descriptors {
              add_descriptor(&mut pool_sizes, descriptor);
            }
          }
        }
        PipelineType::Compute(c) => {
          descriptor_count += c.descriptor_sets.len();
          for descriptor_set in &c.descriptor_sets {
            for descriptor in &descriptor_set.descriptors {
              add_descriptor(&mut pool_sizes, descriptor);
            }
          }
        }
      }
    }

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::default()
      .max_sets(descriptor_count as u32)
      .pool_sizes(&pool_sizes);
    let descriptor_pool =
      unsafe { logical_device.create_descriptor_pool(&descriptor_pool_create_info, None)? };

    let default_desc_config = vec![default_descriptor_set];
    let (default_desc_layouts, default_descs, default_buffers) =
      Pipeline::get_descriptor_set_layouts(
        &default_desc_config,
        descriptor_pool,
        logical_device,
        memory_manager,
      )?;

    let mut vk_pipelines = Vec::new();
    let mut i = 0;
    for pipeline in pipelines {
      match pipeline {
        PipelineType::Graphics(config) => {
          vk_pipelines.push((
            config.name.clone(),
            Pipeline::init_graphics_pipeline(
              logical_device,
              render_pass,
              config,
              descriptor_pool,
              memory_manager,
              swap_chain_extent,
              i,
              &default_descs,
              &default_desc_layouts,
            )?,
          ));
          i += 1;
        }
        PipelineType::Compute(config) => {
          vk_pipelines.push((
            config.name.clone(),
            Pipeline::init_compute_pipeline(
              logical_device,
              config,
              descriptor_pool,
              memory_manager,
            )?,
          ));
        }
      }
    }

    Ok(Self {
      pipelines: vk_pipelines,
      descriptor_pool,
      logical_device: logical_device.clone(),
      default_desc_layouts,
      default_buffers,
    })
  }

  pub fn destroy(&mut self) {
    unsafe {
      self
        .logical_device
        .destroy_descriptor_pool(self.descriptor_pool, None);
    }
    for layout in &self.default_desc_layouts {
      unsafe {
        self
          .logical_device
          .destroy_descriptor_set_layout(*layout, None);
      }
    }

    std::fs::create_dir_all("cache").unwrap();
    for (_, pipeline) in &mut self.pipelines {
      pipeline.destroy(&self.logical_device);
    }
  }

  pub fn get_pipeline(&self, name: &str) -> Option<&Pipeline> {
    Some(&self.pipelines.iter().find(|(n, _)| n == name)?.1)
  }

  pub fn update_descriptor<T: Sized>(
    &self,
    memory_manager: &mut MemoryManager,
    mem: &BufferMemory,
    data: &[T],
  ) -> Option<()> {
    memory_manager.write_to_buffer(mem, data);

    Some(())
  }

  pub fn create_descriptor_mem(
    &self,
    memory_manager: &mut MemoryManager,
    pipeline_name: &str,
    descriptor_set: usize,
    descriptor: usize,
    size: usize,
  ) -> Option<BufferMemory> {
    let pipeline = self
      .pipelines
      .iter()
      .find(|(n, _)| n == pipeline_name)
      .map(|(_, p)| p)?;
    let set = pipeline.descriptor_buffers.get(&descriptor_set)?;
    let desc = set.get(&descriptor)?;

    Some(memory_manager.reserve_buffer_mem(*desc, size)?.0)
  }

  pub fn pipeline_names(&self) -> Vec<&String> {
    self.pipelines.iter().map(|(n, _)| n).collect()
  }

  pub fn update_camera(&mut self, memory_manager: &mut MemoryManager, camera: &Camera) {
    let regions = [vk::BufferCopy {
      dst_offset: 0,
      src_offset: 0,
      size: 128,
    }];
    let data = [camera.view_matrix(), camera.projection_matrix()];
    memory_manager.write_to_buffer_direct(self.default_buffers[&0][&0], &data, &regions);
  }
}

pub struct Pipeline {
  name: String,
  pipeline: vk::Pipeline,
  pipeline_layout: vk::PipelineLayout,
  pipeline_bind_point: vk::PipelineBindPoint,
  descriptor_sets: Vec<vk::DescriptorSet>,
  descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
  descriptor_buffers: HashMap<usize, HashMap<usize, BufferId>>,
  cache: vk::PipelineCache,
}

impl Pipeline {
  pub fn default_shader() -> GraphicsPipelineConfig<'static> {
    GraphicsPipelineConfig::new("default".to_string())
      .set_frag_shader(vk_shader_macros::include_glsl!("./assets/shader.frag").to_vec())
      .add_descriptor_set(
        DescriptorSet::default().add_descriptor(DescriptorType::new_storage(
          vk::ShaderStageFlags::FRAGMENT,
          144,
        )),
      )
  }

  pub fn init_compute_pipeline(
    logical_device: &ash::Device,
    pipeline: &ComputePipelineConfig,
    descriptor_pool: vk::DescriptorPool,
    memory_manager: &mut MemoryManager,
  ) -> Result<Self, Error> {
    let main_function_name = std::ffi::CString::new("main").unwrap();

    let shader_create_info = vk::ShaderModuleCreateInfo::default().code(&pipeline.shader);
    let shader_module = unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;

    let shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
      .stage(vk::ShaderStageFlags::COMPUTE)
      .module(shader_module)
      .name(&main_function_name);

    let (descriptor_layouts, descriptor_sets, descriptor_buffers) =
      Self::get_descriptor_set_layouts(
        &pipeline.descriptor_sets,
        descriptor_pool,
        logical_device,
        memory_manager,
      )?;

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
      pipeline_bind_point: vk::PipelineBindPoint::COMPUTE,
      descriptor_sets,
      descriptor_set_layouts: descriptor_layouts.to_vec(),
      descriptor_buffers,
      cache: pipeline_cache,
    })
  }

  #[allow(clippy::too_many_arguments)]
  pub fn init_graphics_pipeline(
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
    pipeline: &GraphicsPipelineConfig,
    descriptor_pool: vk::DescriptorPool,
    memory_manager: &mut MemoryManager,
    swapchain_extend: &vk::Extent2D,
    subpass: u32,
    default_descs: &[vk::DescriptorSet],
    default_desc_layouts: &[vk::DescriptorSetLayout],
  ) -> Result<Self, Error> {
    let main_function_name = std::ffi::CString::new("main").unwrap();

    let mut shader_modules = vec![];

    let shader_create_info = vk::ShaderModuleCreateInfo::default()
      .code(vk_shader_macros::include_glsl!("./assets/shader.vert"));
    let shader_module = unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
    shader_modules.push((shader_module, vk::ShaderStageFlags::VERTEX));

    if let Some(shader) = &pipeline.geo_shader {
      let shader_create_info = vk::ShaderModuleCreateInfo::default().code(shader);
      let shader_module =
        unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
      shader_modules.push((shader_module, vk::ShaderStageFlags::GEOMETRY));
    }

    let shader = &pipeline.frag_shader;
    let shader_create_info = vk::ShaderModuleCreateInfo::default().code(shader);
    let shader_module = unsafe { logical_device.create_shader_module(&shader_create_info, None) }?;
    shader_modules.push((shader_module, vk::ShaderStageFlags::FRAGMENT));

    let mut shader_stages = vec![];
    for shader in &shader_modules {
      let shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
        .stage(shader.1)
        .module(shader.0)
        .name(&main_function_name);
      shader_stages.push(shader_stage_create_info);
    }

    let vertex_binding_descs = [
      vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(32)
        .input_rate(vk::VertexInputRate::VERTEX),
      vk::VertexInputBindingDescription::default()
        .binding(1)
        .stride(152)
        .input_rate(vk::VertexInputRate::INSTANCE),
    ];

    let mut vertex_attrib_descs = vec![];

    for i in 0..2 {
      vertex_attrib_descs.push(
        vk::VertexInputAttributeDescription::default()
          .binding(0)
          .location(i)
          .offset(i * 12)
          .format(vk::Format::R32G32B32_SFLOAT),
      );
    }

    vertex_attrib_descs.push(
      vk::VertexInputAttributeDescription::default()
        .binding(0)
        .location(2)
        .offset(24)
        .format(vk::Format::R32G32_SFLOAT),
    );

    for i in 0..8 {
      vertex_attrib_descs.push(
        vk::VertexInputAttributeDescription::default()
          .binding(1)
          .location(i + 3)
          .offset(i * 16)
          .format(vk::Format::R32G32B32A32_SFLOAT),
      );
    }

    vertex_attrib_descs.push(
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(11)
        .offset(128)
        .format(vk::Format::R32G32B32_SFLOAT),
    );

    for i in 0..2 {
      vertex_attrib_descs.push(
        vk::VertexInputAttributeDescription::default()
          .binding(1)
          .location(12 + i)
          .offset(140 + i * 4)
          .format(vk::Format::R32_SFLOAT),
      );
    }

    vertex_attrib_descs.push(
      vk::VertexInputAttributeDescription::default()
        .binding(1)
        .location(14)
        .offset(148)
        .format(vk::Format::R32_UINT),
    );

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
      .vertex_binding_descriptions(&vertex_binding_descs)
      .vertex_attribute_descriptions(&vertex_attrib_descs);
    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
      .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport_size = (swapchain_extend.width, swapchain_extend.height);

    let viewport = [vk::Viewport::default()
      .x(0.0)
      .y(0.0)
      .width(viewport_size.0 as f32)
      .height(viewport_size.1 as f32)
      .min_depth(0.0)
      .max_depth(1.0)];
    let scissor = [vk::Rect2D::default()
      .offset(vk::Offset2D::default())
      .extent(vk::Extent2D {
        width: viewport_size.0,
        height: viewport_size.1,
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

    let (descriptor_layouts, descriptor_sets, descriptor_buffers) =
      Self::get_descriptor_set_layouts(
        &pipeline.descriptor_sets,
        descriptor_pool,
        logical_device,
        memory_manager,
      )?;

    let descriptor_sets = default_descs
      .iter()
      .copied()
      .chain(descriptor_sets)
      .collect();
    let descriptor_layouts_used: Vec<vk::DescriptorSetLayout> = default_desc_layouts
      .iter()
      .copied()
      .chain(descriptor_layouts.iter().copied())
      .collect();

    let pipeline_layout_create_info =
      vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_layouts_used);
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
      .subpass(subpass);

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
      pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
      descriptor_sets,
      descriptor_set_layouts: descriptor_layouts.to_vec(),
      descriptor_buffers,
      cache: pipeline_cache,
    })
  }

  #[allow(clippy::complexity)]
  fn get_descriptor_set_layouts(
    descriptor_sets_config: &Vec<DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    logical_device: &ash::Device,
    memory_manager: &mut MemoryManager,
  ) -> Result<
    (
      Vec<vk::DescriptorSetLayout>,
      Vec<vk::DescriptorSet>,
      HashMap<usize, HashMap<usize, BufferId>>,
    ),
    Error,
  > {
    let mut descriptor_layouts = vec![];

    for descriptor_set in descriptor_sets_config {
      let mut descriptor_set_layout_binding_descs = vec![];

      for (i, descriptor) in descriptor_set.descriptors.iter().enumerate() {
        match descriptor {
          DescriptorType::StorageBuffer(desc) | DescriptorType::UniformBuffer(desc) => {
            descriptor_set_layout_binding_descs.push(
              vk::DescriptorSetLayoutBinding::default()
                .binding(i as u32)
                .descriptor_type(desc.type_)
                .descriptor_count(1)
                .stage_flags(desc.stage),
            );
          }
          DescriptorType::Image(desc) => {
            descriptor_set_layout_binding_descs.push(
              vk::DescriptorSetLayoutBinding::default()
                .binding(i as u32)
                .descriptor_type(desc.type_)
                .descriptor_count(desc.images.len() as u32)
                .stage_flags(desc.stage),
            );
          }
        }
      }

      let descriptor_set_layout_create_info =
        vk::DescriptorSetLayoutCreateInfo::default().bindings(&descriptor_set_layout_binding_descs);
      let descriptor_set_layout = unsafe {
        logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
      }?;
      descriptor_layouts.push(descriptor_set_layout);
    }

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
      .descriptor_pool(descriptor_pool)
      .set_layouts(&descriptor_layouts);
    let descriptor_sets =
      unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info)? };

    let mut descriptor_buffers = HashMap::new();

    for (j, descriptor_set) in descriptor_sets_config.iter().enumerate() {
      let mut buffers = HashMap::new();

      for (i, descriptor) in descriptor_set.descriptors.iter().enumerate() {
        match descriptor {
          DescriptorType::StorageBuffer(desc) | DescriptorType::UniformBuffer(desc) => {
            let buffer = memory_manager.create_advanced_buffer(
              desc.buffer_usage,
              BufferBlockSize::Exact(desc.size as usize),
            )?;

            buffers.insert(i, buffer);

            let buffer_info_descriptor = [vk::DescriptorBufferInfo::default()
              .buffer(memory_manager.get_vk_buffer(buffer).unwrap())
              .offset(0)
              .range(desc.size)];

            let write_desc_set = vk::WriteDescriptorSet::default()
              .dst_set(descriptor_sets[j])
              .dst_binding(i as u32)
              .descriptor_type(desc.type_)
              .buffer_info(&buffer_info_descriptor);

            unsafe {
              logical_device.update_descriptor_sets(&[write_desc_set], &[]);
            }
          }
          DescriptorType::Image(desc) => {
            if desc.images.is_empty() {
              continue;
            }

            let mut image_infos = Vec::new();
            for image in &desc.images {
              let sampler_image = memory_manager.create_sampler_image(image)?;
              let view = memory_manager.get_vk_image_view(sampler_image).unwrap();
              let sampler = memory_manager.get_vk_sampler(sampler_image).unwrap();

              image_infos.push(
                vk::DescriptorImageInfo::default()
                  .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                  .image_view(view)
                  .sampler(sampler),
              );
            }

            let write_desc_set = vk::WriteDescriptorSet::default()
              .dst_binding(i as u32)
              .dst_set(descriptor_sets[j])
              .descriptor_type(desc.type_)
              .image_info(&image_infos);

            unsafe {
              logical_device.update_descriptor_sets(&[write_desc_set], &[]);
            }
          }
        }
      }

      descriptor_buffers.insert(j, buffers);
    }

    Ok((descriptor_layouts, descriptor_sets, descriptor_buffers))
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

  pub fn destroy(&mut self, logical_device: &ash::Device) {
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

  pub unsafe fn record_command_buffer(
    &self,
    command_buffer: vk::CommandBuffer,
    device: &ash::Device,
  ) {
    device.cmd_bind_pipeline(command_buffer, self.pipeline_bind_point, self.pipeline);
    device.cmd_bind_descriptor_sets(
      command_buffer,
      self.pipeline_bind_point,
      self.pipeline_layout,
      0,
      &self.descriptor_sets,
      &[],
    );
  }
}

fn add_descriptor(pool_sizes: &mut Vec<vk::DescriptorPoolSize>, desc: &DescriptorType) {
  match desc {
    DescriptorType::StorageBuffer(desc) | DescriptorType::UniformBuffer(desc) => {
      internal_add(pool_sizes, desc.type_);
    }
    DescriptorType::Image(desc) => {
      internal_add(pool_sizes, desc.type_);
    }
  }
  fn internal_add(pool_sizes: &mut Vec<vk::DescriptorPoolSize>, ty: vk::DescriptorType) {
    if let Some(pool) = pool_sizes.iter_mut().find(|s| s.ty == ty) {
      pool.descriptor_count += 1;
    } else {
      pool_sizes.push(vk::DescriptorPoolSize::default().ty(ty).descriptor_count(1));
    }
  }
}
