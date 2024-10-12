use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use super::{allocator::{Allocator, SimpleBufferMemory}, buffer::Buffer, manager::SimpleBufferId};

pub struct SimpleBuffer {
  id: SimpleBufferId,
  buffer: Buffer,
  allocator: Allocator,
  block_size: usize,
}

impl SimpleBuffer {
  pub fn new(
    id: SimpleBufferId,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    usage: vk::BufferUsageFlags,
    block_size: usize,
  ) -> Result<Self, Error> {
    let buffer = Buffer::new(
      allocator,
      device,
      block_size,
      usage,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;

    let allocator = Allocator::new(block_size);

    Ok(Self {
      id,
      buffer,
      allocator,
      block_size,
    })
  }

  pub fn cleanup(
    self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    unsafe { self.buffer.cleanup(device, allocator) }
  }

  pub fn resize_buffer(
    &mut self,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    required_size: usize,
  ) -> Result<(), Error> {
    let size = (required_size as f32 / self.block_size as f32).ceil() as usize * self.block_size;
    let mut new_buffer = Buffer::new(
      allocator,
      device,
      size,
      self.buffer.usage(),
      self.buffer.location(),
    )?;

    let old_ptr = unsafe { self.buffer.ptr().unwrap() };
    new_buffer.write(old_ptr, self.buffer.size(), 0)?;

    let old_buffer = std::mem::replace(&mut self.buffer, new_buffer);
    unsafe { old_buffer.cleanup(device, allocator)? };

    Ok(())
  }

  pub fn add_to_buffer<T>(
    &mut self,
    data: &[T],
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Option<(SimpleBufferMemory, bool)> {
    let size = std::mem::size_of_val(data);
    let (mem, buffer_resized) = self.reserve_buffer_mem(size, device, allocator)?;

    self.write_to_buffer(&mem, data)?;

    Some((mem, buffer_resized))
  }

  pub fn write_to_buffer<T>(&mut self, mem: &SimpleBufferMemory, data: &[T]) -> Option<()> {
    let size = std::mem::size_of_val(data);
    let regions = [vk::BufferCopy {
      src_offset: 0,
      dst_offset: mem.offset() as u64,
      size: size as u64,
    }];
    self.write_to_buffer_direct(data, &regions)
  }

  pub fn write_to_buffer_direct<T: Sized>(
    &mut self,
    data: &[T],
    regions: &[vk::BufferCopy],
  ) -> Option<()> {
    let data_ptr = data.as_ptr() as *const u8;
    for copy in regions {
      self
        .buffer
        .write(
          unsafe { data_ptr.byte_add(copy.src_offset as usize) },
          copy.size as usize,
          copy.dst_offset as usize,
        )
        .ok()?;
    }

    Some(())
  }

  pub fn reserve_buffer_mem(
    &mut self,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Option<(SimpleBufferMemory, bool)> {
    if let Some(mem) = self.allocator.alloc_simple(size, self.id) {
      Some((mem, false))
    } else {
      self
        .resize_buffer(allocator, device, size + self.buffer.size())
        .ok()?;

      let mem = self.allocator.alloc_simple(size, self.id)?;

      Some((mem, true))
    }
  }

  pub fn resize_buffer_mem(
    &mut self,
    mem: &mut SimpleBufferMemory,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Option<bool> {
    let (new_mem, buffer_resized) = self.reserve_buffer_mem(size, device, allocator)?;

    let ptr = unsafe { self.buffer.ptr().unwrap().byte_add(mem.offset()) };
    self.buffer.write(ptr, mem.size(), new_mem.offset()).ok()?;

    let old_mem = std::mem::replace(mem, new_mem);
    self.free_buffer_mem(old_mem);

    Some(buffer_resized)
  }

  pub fn free_buffer_mem(&mut self, mem: SimpleBufferMemory) {
    self.allocator.free(mem.offset(), mem.size());
  }

  pub fn size(&self) -> usize {
    self.buffer.size()
  }

  pub fn vk_buffer(&self) -> ash::vk::Buffer {
    self.buffer.buffer()
  }
}
