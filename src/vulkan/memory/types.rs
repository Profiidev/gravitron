use anyhow::Error;
use gpu_allocator::vulkan;

use crate::Id;

use super::{
  advanced_buffer::AdvancedBuffer, image::Image, sampler_image::SamplerImage,
  simple_buffer::SimpleBuffer,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BufferId {
  Advanced(Id),
  Simple(Id),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImageId {
  Simple(Id),
  Sampler(Id),
}

pub const BUFFER_BLOCK_SIZE_LARGE: usize = 1024 * 1024 * 64;
pub const BUFFER_BLOCK_SIZE_MEDIUM: usize = 1024 * 64;
pub const BUFFER_BLOCK_SIZE_SMALL: usize = 64;

pub enum BufferBlockSize {
  Large,
  Medium,
  Small,
  Exact(usize),
}

pub enum BufferType {
  Simple(SimpleBuffer),
  Advanced(AdvancedBuffer),
}

pub enum ImageType {
  Simple(Image),
  Sampler(SamplerImage),
}

impl BufferType {
  pub fn cleanup(
    self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    match self {
      BufferType::Advanced(buffer) => buffer.cleanup(device, allocator),
      BufferType::Simple(buffer) => buffer.cleanup(device, allocator),
    }
  }
}

impl ImageType {
  pub fn cleanup(
    self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    match self {
      ImageType::Sampler(sampler_image) => sampler_image.cleanup(device, allocator),
      ImageType::Simple(image) => image.cleanup(device, allocator),
    }
  }
}
