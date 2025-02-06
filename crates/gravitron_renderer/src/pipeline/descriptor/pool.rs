use std::collections::HashMap;

use anyhow::Error;
use ash::vk;

const DEFAULT_COUNT: u32 = 20;
const TYPES: [vk::DescriptorType; 4] = [
  vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
  vk::DescriptorType::STORAGE_IMAGE,
  vk::DescriptorType::STORAGE_BUFFER,
  vk::DescriptorType::UNIFORM_BUFFER,
];

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DescriptorPoolHandle(pub(crate) u64);

pub struct DescriptorPool {
  #[allow(unused)]
  id: DescriptorPoolHandle,
  pool: vk::DescriptorPool,
  size_left: HashMap<vk::DescriptorType, u32>,
  set_left: u32,
}

impl DescriptorPool {
  pub fn new(
    min_space: HashMap<vk::DescriptorType, u32>,
    id: DescriptorPoolHandle,
    logical_device: &ash::Device,
  ) -> Result<Self, Error> {
    let mut pool_sizes = Vec::new();
    for r#type in TYPES {
      let size = min_space.get(&r#type).unwrap_or(&DEFAULT_COUNT);
      pool_sizes.push(
        vk::DescriptorPoolSize::default()
          .ty(r#type)
          .descriptor_count(*size),
      );
    }

    let info = vk::DescriptorPoolCreateInfo::default()
      .max_sets(DEFAULT_COUNT / 2)
      .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
      .pool_sizes(&pool_sizes);

    let pool = unsafe { logical_device.create_descriptor_pool(&info, None) }?;

    Ok(Self {
      id,
      size_left: min_space,
      set_left: DEFAULT_COUNT / 2,
      pool,
    })
  }

  #[inline]
  pub fn has_space(&self, needed: &HashMap<vk::DescriptorType, u32>) -> bool {
    needed
      .iter()
      .all(|(r#type, size)| self.size_left.get(r#type).unwrap_or(&0) >= size)
      && self.set_left > 0
  }

  #[inline]
  pub fn add_set(&mut self, needed: &HashMap<vk::DescriptorType, u32>) -> vk::DescriptorPool {
    for (r#type, size) in needed {
      *self.size_left.entry(*r#type).or_default() -= size;
    }
    self.set_left -= 1;

    self.pool
  }

  pub fn cleanup(&self, logical_device: &ash::Device) {
    unsafe { logical_device.destroy_descriptor_pool(self.pool, None) };
  }
}
