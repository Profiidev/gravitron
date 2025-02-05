use std::collections::HashMap;

use anyhow::Error;
use ash::vk;

use crate::memory::MemoryManager;

use super::{
  pool::{DescriptorPool, DescriptorPoolId},
  Descriptor, DescriptorId, DescriptorInfo, DescriptorMut, DescriptorRef, DescriptorSet,
  DescriptorSetId, DescriptorType,
};

pub struct DescriptorManager {
  logical_device: ash::Device,
  max_descriptor_set_id: u64,
  max_descriptor_id: u64,
  max_pool_id: u64,
  descriptor_sets: HashMap<DescriptorSetId, DescriptorSet>,
  descriptor_pools: HashMap<DescriptorPoolId, DescriptorPool>,
  changed: bool,
}

impl DescriptorManager {
  pub(crate) fn new(logical_device: &ash::Device) -> Result<Self, Error> {
    let id = DescriptorPoolId(0);
    let pool = DescriptorPool::new(Default::default(), id, logical_device)?;
    let mut pools = HashMap::new();
    pools.insert(id, pool);

    Ok(DescriptorManager {
      logical_device: logical_device.clone(),
      max_descriptor_id: 0,
      max_descriptor_set_id: 0,
      max_pool_id: 1,
      descriptor_sets: Default::default(),
      descriptor_pools: pools,
      changed: false,
    })
  }

  pub fn create_descriptor_set(
    &mut self,
    descriptor: Vec<DescriptorInfo>,
    memory_manager: &MemoryManager,
  ) -> Option<(DescriptorSetId, Vec<DescriptorId>)> {
    let mut descriptors = HashMap::new();
    let mut bind_desc = Vec::new();
    let mut size_needed = HashMap::new();
    let mut flags = Vec::new();

    for (i, info) in descriptor.iter().enumerate() {
      bind_desc.push(
        vk::DescriptorSetLayoutBinding::default()
          .binding(i as u32)
          .stage_flags(info.stage)
          .descriptor_type(info.r#type.vk_type())
          .descriptor_count(info.r#type.count()),
      );

      *size_needed.entry(info.r#type.vk_type()).or_default() += info.r#type.count();

      match info.r#type {
        DescriptorType::StorageBuffer(_) | DescriptorType::UniformBuffer(_) => {
          flags.push(vk::DescriptorBindingFlags::UPDATE_AFTER_BIND)
        }
        _ => flags.push(vk::DescriptorBindingFlags::empty()),
      }
    }

    let mut flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&flags);
    let layout_create_info = vk::DescriptorSetLayoutCreateInfo::default()
      .bindings(&bind_desc)
      .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
      .push_next(&mut flags);
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

    let mut descriptor_ids = Vec::new();
    for (i, info) in descriptor.into_iter().enumerate() {
      write_descriptor(
        &self.logical_device,
        &info.r#type,
        i as u32,
        set,
        memory_manager,
      );

      let id = DescriptorId(self.max_descriptor_id);
      descriptor_ids.push(id);
      descriptors.insert(
        id,
        Descriptor {
          id,
          binding: i as u32,
          previous: None,
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

    Some((id, descriptor_ids))
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

  #[inline]
  pub fn descriptor(&self, id: DescriptorId) -> Option<DescriptorRef<'_>> {
    self
      .descriptor_sets
      .values()
      .find_map(|set| set.descriptors.get(&id))
      .map(DescriptorRef)
  }

  #[inline]
  pub fn descriptor_mut(&mut self, id: DescriptorId) -> Option<DescriptorMut<'_>> {
    self
      .descriptor_sets
      .values_mut()
      .find_map(|set| set.descriptors.get_mut(&id))
      .map(DescriptorMut)
  }

  pub(crate) fn update_changed(&mut self, memory_manager: &MemoryManager) {
    for set in self.descriptor_sets.values() {
      for descriptor in set.descriptors.values() {
        if let Some(prev) = &descriptor.previous {
          if *prev != descriptor.r#type {
            self.changed = true;

            write_descriptor(
              &self.logical_device,
              &descriptor.r#type,
              descriptor.binding,
              set.set,
              memory_manager,
            );
          }
        }
      }
    }
  }

  #[inline]
  pub(crate) fn descriptor_changed(&self) -> bool {
    self.changed
  }

  #[inline]
  pub(crate) fn reset_changed(&mut self) {
    self.changed = false;
    for set in self.descriptor_sets.values_mut() {
      for descriptor in set.descriptors.values_mut() {
        descriptor.previous = None;
      }
    }
  }
}

fn write_descriptor(
  logical_device: &ash::Device,
  r#type: &DescriptorType,
  binding: u32,
  set: vk::DescriptorSet,
  memory_manager: &MemoryManager,
) {
  match r#type {
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
        .dst_binding(binding)
        .descriptor_type(r#type.vk_type())
        .buffer_info(&buffer_info);

      unsafe {
        logical_device.update_descriptor_sets(&[write], &[]);
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

        if let DescriptorType::Sampler(_) = r#type {
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
        .dst_binding(binding)
        .descriptor_type(r#type.vk_type())
        .image_info(&image_infos);

      unsafe {
        logical_device.update_descriptor_sets(&[write], &[]);
      }
    }
    DescriptorType::InputAttachment(image) => {
      let image_info = [vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(
          memory_manager
            .get_vk_image_view(*image)
            .expect("Failed to get ImageView"),
        )];

      let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(binding)
        .descriptor_type(r#type.vk_type())
        .image_info(&image_info);

      unsafe {
        logical_device.update_descriptor_sets(&[write], &[]);
      }
    }
  }
}
