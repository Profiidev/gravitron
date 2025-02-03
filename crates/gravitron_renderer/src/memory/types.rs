use anyhow::Error;
use gpu_allocator::vulkan;

use super::{
  advanced_buffer::AdvancedBuffer, image::Image, sampler_image::SamplerImage,
  simple_buffer::SimpleBuffer,
};

pub use super::allocator::BufferMemory;
pub use ash::vk::{BufferCopy, BufferUsageFlags};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BufferId {
  Advanced(u64),
  Simple(u64),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImageId {
  Simple(u64),
  Sampler(u64),
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
  pub(crate) fn cleanup(
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
  pub(crate) fn cleanup(
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

pub enum BufferMemoryLocation {
  GpuToCpu,
  CpuToGpu,
}

impl From<BufferMemoryLocation> for gpu_allocator::MemoryLocation {
  fn from(value: BufferMemoryLocation) -> Self {
    match value {
      BufferMemoryLocation::CpuToGpu => gpu_allocator::MemoryLocation::CpuToGpu,
      BufferMemoryLocation::GpuToCpu => gpu_allocator::MemoryLocation::GpuToCpu,
    }
  }
}
