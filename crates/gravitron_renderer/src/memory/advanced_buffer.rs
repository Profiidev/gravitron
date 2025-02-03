use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use crate::memory::error::MemoryError;

use super::{
  allocator::{Allocator, BufferMemory},
  buffer::{buffer_copy, buffer_copy_info, Buffer},
  manager::Transfer,
  types::BufferId,
};

pub struct AdvancedBuffer {
  id: BufferId,
  transfer: Buffer,
  gpu: Buffer,
  allocator: Allocator,
  block_size: usize,
  reallocated: bool,
}

impl AdvancedBuffer {
  pub fn new(
    id: BufferId,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    usage: vk::BufferUsageFlags,
    block_size: usize,
  ) -> Result<Self, Error> {
    let transfer = Buffer::new(
      allocator,
      device,
      block_size,
      usage | vk::BufferUsageFlags::TRANSFER_SRC,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;

    let gpu = Buffer::new(
      allocator,
      device,
      block_size,
      usage | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC,
      gpu_allocator::MemoryLocation::GpuOnly,
    )?;

    let allocator = Allocator::new(block_size);

    Ok(Self {
      id,
      transfer,
      gpu,
      allocator,
      block_size,
      reallocated: false,
    })
  }

  fn resize_transfer_buffer(
    &mut self,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) {
    if self.transfer.size() < size {
      let new_size = (size as f32 / self.block_size as f32).ceil() as usize * self.block_size;
      self.transfer.resize(new_size, device, allocator).ok();
    }
  }

  pub fn cleanup(
    self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    unsafe {
      self.transfer.cleanup(device, allocator)?;
      self.gpu.cleanup(device, allocator)
    }
  }

  pub fn add_to_buffer<T: Sized>(
    &mut self,
    data: &[T],
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Result<BufferMemory, Error> {
    let size = std::mem::size_of_val(data);
    let mem = self
      .reserve_buffer_mem(size, device, allocator, transfer)
      .ok_or(MemoryError::Reallocate)?;

    self.write_to_buffer(&mem, data, device, allocator, transfer)?;

    Ok(mem)
  }

  pub fn write_to_buffer<T: Sized>(
    &mut self,
    mem: &BufferMemory,
    data: &[T],
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Result<(), Error> {
    let size = std::mem::size_of_val(data);
    assert!(size <= mem.size());
    let regions = buffer_copy_info(mem.offset(), size);
    self.write_to_buffer_direct(data, &regions, device, allocator, transfer)
  }

  pub fn write_to_buffer_direct<T: Sized>(
    &mut self,
    data: &[T],
    regions: &[vk::BufferCopy],
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Result<(), Error> {
    let size = std::mem::size_of_val(data);
    self.resize_transfer_buffer(size, device, allocator);
    self.transfer.fill(data)?;

    transfer.reset(device)?;

    buffer_copy(
      &self.transfer,
      &self.gpu,
      device,
      transfer.queue(),
      transfer,
      regions,
    )?;

    Ok(())
  }

  pub fn reserve_buffer_mem(
    &mut self,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Option<BufferMemory> {
    if let Some(mem) = self.allocator.alloc(size, self.id) {
      Some(mem)
    } else {
      let additional_size =
        (size as f32 / self.block_size as f32).ceil() as usize * self.block_size;
      let new_gpu = Buffer::new(
        allocator,
        device,
        self.gpu.size() + additional_size,
        self.gpu.usage(),
        self.gpu.location(),
      )
      .ok()?;

      transfer.reset(device).ok()?;

      let regions = buffer_copy_info(0, self.gpu.size());
      buffer_copy(
        &self.gpu,
        &new_gpu,
        device,
        transfer.queue(),
        transfer,
        &regions,
      )
      .ok()?;

      transfer.wait(device).ok()?;
      self.reallocated = true;

      let old_gpu = std::mem::replace(&mut self.gpu, new_gpu);
      unsafe { old_gpu.cleanup(device, allocator).ok()? };

      self.allocator.grow(additional_size);

      let mem = self.allocator.alloc(size, self.id).unwrap();

      Some(mem)
    }
  }

  pub fn resize_buffer_mem(
    &mut self,
    mem: &mut BufferMemory,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Result<(), Error> {
    assert!(mem.size() < size);
    let new_mem = self
      .reserve_buffer_mem(size, device, allocator, transfer)
      .ok_or(MemoryError::Reallocate)?;

    transfer.reset(device)?;

    let regions = [vk::BufferCopy {
      src_offset: mem.offset() as u64,
      dst_offset: new_mem.offset() as u64,
      size: mem.size() as u64,
    }];
    buffer_copy(
      &self.gpu,
      &self.gpu,
      device,
      transfer.queue(),
      transfer,
      &regions,
    )?;

    let old_mem = std::mem::replace(mem, new_mem);
    self.free_buffer_mem(old_mem);

    transfer.wait(device)?;

    Ok(())
  }

  pub fn free_buffer_mem(&mut self, mem: BufferMemory) {
    self.allocator.free(mem.offset(), mem.size());
  }

  pub fn vk_buffer(&self) -> vk::Buffer {
    self.gpu.buffer()
  }

  pub fn size(&self) -> usize {
    self.gpu.size()
  }
}
