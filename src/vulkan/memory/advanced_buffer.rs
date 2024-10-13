use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use super::{
  allocator::{Allocator, BufferMemory},
  buffer::{buffer_copy, buffer_copy_info, Buffer},
  manager::Transfer,
};

use crate::Id;

pub struct AdvancedBuffer {
  id: Id,
  transfer: Buffer,
  gpu: Buffer,
  allocator: Allocator,
  block_size: usize,
}

impl AdvancedBuffer {
  pub fn new(
    id: Id,
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
  ) -> Option<(BufferMemory, bool)> {
    let size = std::mem::size_of_val(data);
    let (mem, buffer_resized) = self.reserve_buffer_mem(size, device, allocator, transfer)?;

    self.write_to_buffer(&mem, data, device, allocator, transfer)?;

    Some((mem, buffer_resized))
  }

  pub fn write_to_buffer<T: Sized>(
    &mut self,
    mem: &BufferMemory,
    data: &[T],
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Option<()> {
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
  ) -> Option<()> {
    let size = std::mem::size_of_val(data);
    self.resize_transfer_buffer(size, device, allocator);
    self.transfer.fill(data).ok()?;

    transfer.reset(device).ok()?;

    buffer_copy(
      &self.transfer,
      &self.gpu,
      device,
      transfer.queue(),
      transfer,
      regions,
    )
    .ok()?;

    Some(())
  }

  pub fn reserve_buffer_mem(
    &mut self,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Option<(BufferMemory, bool)> {
    if let Some(mem) = self.allocator.alloc(size, self.id) {
      Some((mem, false))
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

      let old_gpu = std::mem::replace(&mut self.gpu, new_gpu);
      unsafe { old_gpu.cleanup(device, allocator).ok()? };

      self.allocator.grow(additional_size);

      let mem = self.allocator.alloc(size, self.id).unwrap();

      Some((mem, true))
    }
  }

  pub fn resize_buffer_mem(
    &mut self,
    mem: &mut BufferMemory,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    transfer: &Transfer,
  ) -> Option<bool> {
    assert!(mem.size() < size);
    let (new_mem, buffer_resized) = self.reserve_buffer_mem(size, device, allocator, transfer)?;

    transfer.reset(device).ok()?;

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
    )
    .ok()?;

    let old_mem = std::mem::replace(mem, new_mem);
    self.free_buffer_mem(old_mem);

    transfer.wait(device).ok()?;

    Some(buffer_resized)
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
