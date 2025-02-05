use std::{
  collections::HashMap,
  ops::{Deref, DerefMut},
};

use ash::vk;

use pool::DescriptorPoolId;
pub use vk::ShaderStageFlags;

use crate::memory::types::{BufferMemory, ImageId};

pub(crate) mod manager;
mod pool;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DescriptorId(pub(crate) u64);
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DescriptorSetId(pub(crate) u64);

pub(crate) struct DescriptorSet {
  #[allow(unused)]
  id: DescriptorSetId,
  #[allow(unused)]
  pool: DescriptorPoolId,
  set: vk::DescriptorSet,
  layout: vk::DescriptorSetLayout,
  descriptors: HashMap<DescriptorId, Descriptor>,
}

impl DescriptorSet {
  #[inline]
  pub fn set(&self) -> vk::DescriptorSet {
    self.set
  }

  #[inline]
  pub fn layout(&self) -> vk::DescriptorSetLayout {
    self.layout
  }

  pub fn cleanup(&self, logical_device: &ash::Device) {
    unsafe { logical_device.destroy_descriptor_set_layout(self.layout, None) };
  }
}

pub struct Descriptor {
  #[allow(unused)]
  id: DescriptorId,
  previous: Option<DescriptorType>,
  binding: u32,
  r#type: DescriptorType,
}

impl Descriptor {
  #[inline]
  pub fn get_type(&self) -> &DescriptorType {
    &self.r#type
  }
}

pub struct DescriptorInfo {
  pub stage: vk::ShaderStageFlags,
  pub r#type: DescriptorType,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum DescriptorType {
  StorageBuffer(BufferMemory),
  UniformBuffer(BufferMemory),
  Sampler(Vec<ImageId>),
  Image(Vec<ImageId>),
  InputAttachment(ImageId),
}

impl DescriptorType {
  #[inline]
  fn vk_type(&self) -> vk::DescriptorType {
    match self {
      DescriptorType::Image(_) => vk::DescriptorType::STORAGE_IMAGE,
      DescriptorType::Sampler(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      DescriptorType::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
      DescriptorType::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
      DescriptorType::InputAttachment(_) => vk::DescriptorType::INPUT_ATTACHMENT,
    }
  }

  #[inline]
  fn count(&self) -> u32 {
    match self {
      DescriptorType::StorageBuffer(_)
      | DescriptorType::UniformBuffer(_)
      | DescriptorType::InputAttachment(_) => 1,
      DescriptorType::Image(images) | DescriptorType::Sampler(images) => images.len() as u32,
    }
  }

  #[inline]
  pub fn image(&self) -> Option<&[ImageId]> {
    match self {
      DescriptorType::Image(images) => Some(images),
      _ => None,
    }
  }

  #[inline]
  pub fn sampler(&self) -> Option<&[ImageId]> {
    match self {
      DescriptorType::Sampler(images) => Some(images),
      _ => None,
    }
  }

  #[inline]
  pub fn attachment(&self) -> Option<&ImageId> {
    match self {
      DescriptorType::InputAttachment(image) => Some(image),
      _ => None,
    }
  }

  #[inline]
  pub fn storage(&self) -> Option<&BufferMemory> {
    match self {
      DescriptorType::StorageBuffer(buffer) => Some(buffer),
      _ => None,
    }
  }

  #[inline]
  pub fn storage_mut(&mut self) -> Option<&mut BufferMemory> {
    match self {
      DescriptorType::StorageBuffer(buffer) => Some(buffer),
      _ => None,
    }
  }

  #[inline]
  pub fn uniform(&self) -> Option<&BufferMemory> {
    match self {
      DescriptorType::UniformBuffer(buffer) => Some(buffer),
      _ => None,
    }
  }

  #[inline]
  pub fn uniform_mut(&mut self) -> Option<&mut BufferMemory> {
    match self {
      DescriptorType::UniformBuffer(buffer) => Some(buffer),
      _ => None,
    }
  }
}

pub struct DescriptorRef<'d>(&'d Descriptor);

impl Deref for DescriptorRef<'_> {
  type Target = DescriptorType;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0.r#type
  }
}

pub struct DescriptorMut<'d>(&'d mut Descriptor);

impl Deref for DescriptorMut<'_> {
  type Target = DescriptorType;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0.r#type
  }
}

impl DerefMut for DescriptorMut<'_> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    match &self.0.r#type {
      DescriptorType::StorageBuffer(mem) => {
        self.0.previous = Some(DescriptorType::StorageBuffer(unsafe { mem.copy() }))
      }
      DescriptorType::UniformBuffer(mem) => {
        self.0.previous = Some(DescriptorType::UniformBuffer(unsafe { mem.copy() }))
      }
      _ => (),
    }
    &mut self.0.r#type
  }
}
