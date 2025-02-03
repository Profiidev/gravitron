use std::collections::HashMap;

use anyhow::Error;
use ash::vk;

use crate::memory::MemoryManager;

use super::{
  pool::{DescriptorPool, DescriptorPoolId},
  Descriptor, DescriptorId, DescriptorInfo, DescriptorSet, DescriptorSetId, DescriptorType,
};

pub struct DescriptorManager {
  logical_device: ash::Device,
  max_descriptor_set_id: u64,
  max_descriptor_id: u64,
  max_pool_id: u64,
  descriptor_sets: HashMap<DescriptorSetId, DescriptorSet>,
  descriptor_pools: HashMap<DescriptorPoolId, DescriptorPool>,
}

impl DescriptorManager {
  pub(crate) fn new(logical_device: &ash::Device) -> Result<Self, Error> {
    let id = DescriptorPoolId(0);
    let pool = DescriptorPool::new(Default::default(), id, &logical_device)?;
    let mut pools = HashMap::new();
    pools.insert(id, pool);

    Ok(DescriptorManager {
      logical_device: logical_device.clone(),
      max_descriptor_id: 0,
      max_descriptor_set_id: 0,
      max_pool_id: 1,
      descriptor_sets: Default::default(),
      descriptor_pools: pools,
    })
  }

  pub fn create_descriptor_set(
    &mut self,
    descriptor: Vec<DescriptorInfo>,
    memory_manager: &MemoryManager,
  ) -> Option<DescriptorSetId> {
    let mut descriptors = HashMap::new();
    let mut bind_desc = Vec::new();
    let mut size_needed = HashMap::new();

    for (i, info) in descriptor.iter().enumerate() {
      bind_desc.push(
        vk::DescriptorSetLayoutBinding::default()
          .binding(i as u32)
          .stage_flags(info.stage)
          .descriptor_type(info.r#type.vk_type())
          .descriptor_count(info.r#type.count()),
      );

      *size_needed.entry(info.r#type.vk_type()).or_default() += info.r#type.count();
    }

    let layout_create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bind_desc);
    let layout = [unsafe {
      self
        .logical_device
        .create_descriptor_set_layout(&layout_create_info, None)
        .ok()?
    }];

    let (pool_id, pool) = if let Some((id, pool)) = self
      .descriptor_pools
      .iter_mut()
      .find(|(_, pool)| pool.has_space(&size_needed))
    {
      (*id, pool.add_set(&size_needed))
    } else {
      let id = DescriptorPoolId(self.max_pool_id);
      self.max_pool_id += 1;

      let mut pool = DescriptorPool::new(size_needed.clone(), id, &self.logical_device).ok()?;
      let vk_pool = pool.add_set(&size_needed);

      self.descriptor_pools.insert(id, pool);

      (id, vk_pool)
    };

    let allocate_info = vk::DescriptorSetAllocateInfo::default()
      .set_layouts(&layout)
      .descriptor_pool(pool);
    let set = unsafe {
      self
        .logical_device
        .allocate_descriptor_sets(&allocate_info)
        .ok()?
    }[0];

    for (i, info) in descriptor.into_iter().enumerate() {
      match &info.r#type {
        DescriptorType::StorageBuffer(mem) | DescriptorType::UniformBuffer(mem) => {
          let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(
              memory_manager
                .get_vk_buffer(mem.buffer())
                .expect("Invalid Buffer"),
            )
            .offset(mem.offset() as u64)
            .range(mem.size() as u64)];

          let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(i as u32)
            .descriptor_type(info.r#type.vk_type())
            .buffer_info(&buffer_info);

          unsafe {
            self.logical_device.update_descriptor_sets(&[write], &[]);
          }
        }
        DescriptorType::Image(images) | DescriptorType::Sampler(images) => {
          let mut image_infos = Vec::new();
          for image in images {
            let view = memory_manager
              .get_vk_image_view(*image)
              .expect("Failed to get Image View");

            let mut image_info = vk::DescriptorImageInfo::default()
              .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
              .image_view(view);

            if let DescriptorType::Sampler(_) = &info.r#type {
              image_info = image_info.sampler(
                memory_manager
                  .get_vk_sampler(*image)
                  .expect("Sampler Descriptor requires a Sampler Image"),
              );
            }

            image_infos.push(image_info);
          }

          let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(i as u32)
            .descriptor_type(info.r#type.vk_type())
            .image_info(&image_infos);

          unsafe {
            self.logical_device.update_descriptor_sets(&[write], &[]);
          }
        }
      }

      let id = DescriptorId(self.max_descriptor_id);
      descriptors.insert(
        id,
        Descriptor {
          id,
          r#type: info.r#type,
        },
      );
      self.max_descriptor_id += 1;
    }

    let id = DescriptorSetId(self.max_descriptor_set_id);
    let descriptor_set = DescriptorSet {
      id,
      pool: pool_id,
      set,
      layout: layout[0],
      descriptors,
    };
    self.max_descriptor_set_id += 1;

    self.descriptor_sets.insert(id, descriptor_set);

    Some(id)
  }

  pub(crate) fn cleanup(&self) {
    for pool in self.descriptor_pools.values() {
      pool.cleanup(&self.logical_device);
    }
    for set in self.descriptor_sets.values() {
      set.cleanup(&self.logical_device);
    }
  }

  pub(crate) fn vk_layouts(&self, ids: &[DescriptorSetId]) -> Vec<vk::DescriptorSetLayout> {
    ids
      .iter()
      .flat_map(|id| self.descriptor_sets.get(id).map(|set| set.layout()))
      .collect()
  }

  pub(crate) fn vk_sets(&self, ids: &[DescriptorSetId]) -> Vec<vk::DescriptorSet> {
    ids
      .iter()
      .flat_map(|id| self.descriptor_sets.get(id).map(|set| set.set()))
      .collect()
  }
}
