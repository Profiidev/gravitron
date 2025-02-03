use std::collections::HashMap;

use ash::vk;

use pool::DescriptorPoolId;
pub use vk::ShaderStageFlags;

use crate::memory::types::{BufferMemory, ImageId};

pub(crate) mod manager;
mod pool;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DescriptorId(u64);
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DescriptorSetId(u64);

pub(crate) struct DescriptorSet {
  id: DescriptorSetId,
  pool: DescriptorPoolId,
  set: vk::DescriptorSet,
  layout: vk::DescriptorSetLayout,
  descriptors: HashMap<DescriptorId, Descriptor>,
}

impl DescriptorSet {
  pub fn set(&self) -> vk::DescriptorSet {
    self.set
  }

  pub fn layout(&self) -> vk::DescriptorSetLayout {
    self.layout
  }

  pub fn cleanup(&self, logical_device: &ash::Device) {
    unsafe { logical_device.destroy_descriptor_set_layout(self.layout, None) };
  }
}

pub(crate) struct Descriptor {
  id: DescriptorId,
  r#type: DescriptorType,
}

impl Descriptor {
  pub fn get_type(&self) -> &DescriptorType {
    &self.r#type
  }
}

pub struct DescriptorInfo {
  pub stage: vk::ShaderStageFlags,
  pub r#type: DescriptorType,
}

pub enum DescriptorType {
  StorageBuffer(BufferMemory),
  UniformBuffer(BufferMemory),
  Sampler(Vec<ImageId>),
  Image(Vec<ImageId>),
}

impl DescriptorType {
  fn vk_type(&self) -> vk::DescriptorType {
    match self {
      DescriptorType::Image(_) => vk::DescriptorType::STORAGE_IMAGE,
      DescriptorType::Sampler(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      DescriptorType::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
      DescriptorType::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
    }
  }

  fn count(&self) -> u32 {
    match self {
      DescriptorType::StorageBuffer(_) | DescriptorType::UniformBuffer(_) => 1,
      DescriptorType::Image(images) | DescriptorType::Sampler(images) => images.len() as u32,
    }
  }
}
