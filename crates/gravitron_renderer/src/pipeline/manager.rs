use std::collections::HashMap;

use anyhow::Error;
use ash::vk;
use gravitron_plugin::config::vulkan::{DescriptorSet, DescriptorType, ImageConfig, PipelineType};

use crate::{
  ecs::components::camera::Camera,
  graphics::{
    resources::lighting::{LightInfo, PointLight, SpotLight},
    swapchain::SwapChain,
  },
  memory::{manager::MemoryManager, BufferMemory},
};

use super::{
  descriptors::{add_descriptor, get_descriptor_set_layouts},
  Pipeline,
};

pub struct PipelineManager {
  pipelines: Vec<(String, Pipeline)>,
  light_pipeline: Pipeline,
  descriptor_pool: vk::DescriptorPool,
  logical_device: ash::Device,
  default_desc_layouts: Vec<vk::DescriptorSetLayout>,
  default_buffers: HashMap<usize, HashMap<usize, BufferMemory>>,
}

impl PipelineManager {
  pub fn init(
    logical_device: &ash::Device,
    render_pass: vk::RenderPass,
    light_render_pass: vk::RenderPass,
    swapchain: &SwapChain,
    pipelines: &mut Vec<PipelineType>,
    textures: Vec<ImageConfig>,
    memory_manager: &mut MemoryManager,
  ) -> Result<Self, Error> {
    pipelines.push(PipelineType::Graphics(Pipeline::default_shader()));

    let mut descriptor_count = 2;
    let mut pool_sizes = vec![];

    let mut textures_used = vec![ImageConfig::new_bytes(
      include_bytes!("../../../../assets/default.png").to_vec(),
      vk::Filter::LINEAR,
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
      ))
      .add_descriptor(DescriptorType::new_uniform(
        vk::ShaderStageFlags::FRAGMENT,
        64,
      ))
      .add_descriptor(DescriptorType::new_storage(
        vk::ShaderStageFlags::FRAGMENT,
        120,
      ))
      .add_descriptor(DescriptorType::new_storage(
        vk::ShaderStageFlags::FRAGMENT,
        180,
      ));
    for desc in &default_descriptor_set.descriptors {
      add_descriptor(&mut pool_sizes, desc);
    }
    for _ in 0..3 {
      add_descriptor(
        &mut pool_sizes,
        &DescriptorType::new_image(vk::ShaderStageFlags::FRAGMENT, vec![]),
      );
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
    let (default_desc_layouts, default_descs, default_buffers) = get_descriptor_set_layouts(
      &default_desc_config,
      descriptor_pool,
      logical_device,
      memory_manager,
    )?;

    let light_pipeline = Pipeline::light_pipeline(
      "light".into(),
      logical_device,
      swapchain,
      light_render_pass,
      descriptor_pool,
      &default_descs,
      &default_desc_layouts,
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
              &swapchain.get_extent(),
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
      light_pipeline,
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
    self.light_pipeline.destroy(&self.logical_device);
  }

  pub fn get_pipeline(&self, name: &str) -> Option<&Pipeline> {
    Some(&self.pipelines.iter().find(|(n, _)| n == name)?.1)
  }

  pub fn get_light_pipeline(&self) -> &Pipeline {
    &self.light_pipeline
  }

  pub fn update_descriptor<T: Sized>(
    &self,
    memory_manager: &mut MemoryManager,
    mem: &BufferMemory,
    data: &[T],
  ) {
    memory_manager.write_to_buffer(mem, data);
  }

  pub fn get_descriptor_mem(
    &mut self,
    pipeline_name: &str,
    descriptor_set: usize,
    descriptor: usize,
  ) -> Option<BufferMemory> {
    let pipeline = self
      .pipelines
      .iter_mut()
      .find(|(n, _)| n == pipeline_name)
      .map(|(_, p)| p)?;
    let set = pipeline.descriptor_buffers.get_mut(&descriptor_set)?;
    let desc = set.get_mut(&descriptor)?;

    desc.take()
  }

  pub fn pipeline_names(&self) -> Vec<&String> {
    self.pipelines.iter().map(|(n, _)| n).collect()
  }

  pub fn update_camera(&mut self, memory_manager: &mut MemoryManager, camera: &Camera) {
    let data = [camera.view_matrix(), camera.projection_matrix()];
    memory_manager.write_to_buffer(&self.default_buffers[&0][&0], &data);
  }

  pub fn update_lights(
    &mut self,
    memory_manager: &mut MemoryManager,
    light_info: LightInfo,
    pls: &[PointLight],
    sls: &[SpotLight],
  ) -> Option<bool> {
    let mut resized = false;
    let pls_size = std::mem::size_of_val(pls);
    {
      let pls_mem = self
        .default_buffers
        .get_mut(&0)
        .unwrap()
        .get_mut(&3)
        .unwrap();

      if pls_mem.size() <= pls_size {
        resized = memory_manager.resize_buffer_mem(pls_mem, pls_size)? || resized;
      }
    }

    let sls_size = std::mem::size_of_val(sls);
    {
      let sls_mem = self
        .default_buffers
        .get_mut(&0)
        .unwrap()
        .get_mut(&4)
        .unwrap();

      if sls_mem.size() <= sls_size {
        resized = memory_manager.resize_buffer_mem(sls_mem, sls_size)? || resized;
      }
    }

    let light_info_mem = &self.default_buffers[&0][&2];
    memory_manager.write_to_buffer(light_info_mem, &[light_info]);

    if pls_size > 0 {
      let pls_mem = &self.default_buffers[&0][&3];
      memory_manager.write_to_buffer(pls_mem, pls);
    }

    if sls_size > 0 {
      let sls_mem = &self.default_buffers[&0][&4];
      memory_manager.write_to_buffer(sls_mem, sls);
    }

    Some(resized)
  }
}
