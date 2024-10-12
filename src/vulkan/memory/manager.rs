use std::{collections::HashMap, mem::ManuallyDrop};

use anyhow::Error;
use ash::vk::{self, ImageViewCreateInfo};
use gpu_allocator::vulkan;

use crate::vulkan::{
  device::Device,
  instance::InstanceDevice,
  pipeline::pools::{CommandBufferType, Pools},
};

use super::{allocator::BufferMemory, image::Image, manged_buffer::ManagedBuffer};

pub type BufferId = crate::Id;
pub type ImageId = crate::Id;

pub const BUFFER_BLOCK_SIZE_LARGE: usize = 1024 * 1024 * 64;
pub const BUFFER_BLOCK_SIZE_MEDIUM: usize = 1024 * 64;
pub const BUFFER_BLOCK_SIZE_SMALL: usize = 64;

pub enum BufferBlockSize {
  Large,
  Medium,
  Small,
  Exact(usize),
}

pub struct MemoryManager {
  buffers: HashMap<BufferId, ManagedBuffer>,
  buffer_used: HashMap<BufferId, vk::Fence>,
  last_buffer_id: BufferId,
  images: HashMap<ImageId, Image>,
  last_image_id: BufferId,
  allocator: ManuallyDrop<vulkan::Allocator>,
  device: ash::Device,
  command_buffers: Vec<vk::CommandBuffer>,
  fences: Vec<vk::Fence>,
  transfer_queue: vk::Queue,
}

impl MemoryManager {
  pub fn new(instance: &InstanceDevice, device: &Device, pools: &mut Pools) -> Result<Self, Error> {
    let logical_device = device.get_device();

    let allocator = vulkan::Allocator::new(&vulkan::AllocatorCreateDesc {
      device: logical_device.clone(),
      physical_device: instance.get_physical_device(),
      instance: instance.get_instance().clone(),
      debug_settings: Default::default(),
      buffer_device_address: false,
      allocation_sizes: Default::default(),
    })?;

    let command_buffers =
      pools.create_command_buffers(logical_device, 5, CommandBufferType::Transfer)?;

    let fence_create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let mut fences = Vec::new();
    for _ in 0..command_buffers.len() {
      fences.push(unsafe { logical_device.create_fence(&fence_create_info, None)? });
    }

    Ok(Self {
      buffers: HashMap::new(),
      buffer_used: HashMap::new(),
      last_buffer_id: 0,
      images: HashMap::new(),
      last_image_id: 0,
      allocator: ManuallyDrop::new(allocator),
      device: logical_device.clone(),
      command_buffers,
      fences,
      transfer_queue: device.get_queues().transfer(),
    })
  }

  pub fn create_buffer(
    &mut self,
    usage: vk::BufferUsageFlags,
    block_size: BufferBlockSize,
  ) -> Result<BufferId, Error> {
    let id = self.last_buffer_id;
    let buffer = ManagedBuffer::new(
      id,
      &mut self.allocator,
      &self.device,
      usage,
      block_size.into(),
    )?;

    self.buffers.insert(id, buffer);
    self.last_buffer_id += 1;
    Ok(id)
  }

  pub fn create_image(
    &mut self,
    location: gpu_allocator::MemoryLocation,
    image_info: &vk::ImageCreateInfo,
    image_view_info: &ImageViewCreateInfo,
  ) -> Result<ImageId, Error> {
    let image = Image::new(
      &self.device,
      &mut self.allocator,
      location,
      image_info,
      image_view_info,
    )?;

    self.images.insert(self.last_image_id, image);
    let id = self.last_image_id;
    self.last_image_id += 1;
    Ok(id)
  }

  pub fn reserve_buffer_mem(
    &mut self,
    buffer_id: BufferId,
    size: usize,
  ) -> Option<(BufferMemory, bool)> {
    let transfer = self.reserve_transfer(buffer_id).ok()?;
    let buffer = self.buffers.get_mut(&buffer_id)?;

    buffer.reserve_buffer_mem(
      size,
      &self.device,
      &mut self.allocator,
      self.transfer_queue,
      &transfer,
    )
  }

  pub fn add_to_buffer<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
  ) -> Option<(BufferMemory, bool)> {
    let transfer = self.reserve_transfer(buffer_id).ok()?;
    let buffer = self.buffers.get_mut(&buffer_id)?;

    let mem = buffer.add_to_buffer(
      data,
      &self.device,
      &mut self.allocator,
      self.transfer_queue,
      &transfer,
    );

    self.buffer_used.insert(buffer_id, transfer.fence);
    mem
  }

  pub fn write_to_buffer<T: Sized>(&mut self, mem: &BufferMemory, data: &[T]) -> Option<()> {
    let transfer = self.reserve_transfer(mem.buffer()).ok()?;
    let buffer = self.buffers.get_mut(&mem.buffer())?;

    buffer.write_to_buffer(
      mem,
      data,
      &self.device,
      &mut self.allocator,
      self.transfer_queue,
      &transfer,
    )?;

    self.buffer_used.insert(mem.buffer(), transfer.fence);
    Some(())
  }

  pub fn write_to_buffer_direct<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
    regions: &[vk::BufferCopy],
  ) -> Option<()> {
    let transfer = self.reserve_transfer(buffer_id).ok()?;
    let buffer = self.buffers.get_mut(&buffer_id)?;

    buffer.write_to_buffer_direct(
      data,
      regions,
      &self.device,
      &mut self.allocator,
      self.transfer_queue,
      &transfer,
    )?;

    self.buffer_used.insert(buffer_id, transfer.fence);
    Some(())
  }

  pub fn resize_buffer_mem(&mut self, mem: &mut BufferMemory, size: usize) -> Option<bool> {
    let transfer = self.reserve_transfer(mem.buffer()).ok()?;
    let buffer = self.buffers.get_mut(&mem.buffer())?;

    buffer.resize_buffer_mem(
      mem,
      size,
      &self.device,
      &mut self.allocator,
      self.transfer_queue,
      &transfer,
    )
  }

  pub fn free_buffer_mem(&mut self, mem: BufferMemory) {
    let buffer = self.buffers.get_mut(&mem.buffer()).unwrap();
    buffer.free_buffer_mem(mem);
  }

  pub fn get_vk_buffer(&self, buffer_id: BufferId) -> Option<vk::Buffer> {
    Some(self.buffers.get(&buffer_id)?.vk_buffer())
  }

  pub fn get_vk_image_view(&self, image_id: ImageId) -> Option<vk::ImageView> {
    Some(self.images.get(&image_id)?.image_view())
  }

  pub fn get_buffer_size(&self, buffer_id: BufferId) -> Option<usize> {
    Some(self.buffers.get(&buffer_id)?.size())
  }

  fn reserve_transfer(&mut self, buffer_id: BufferId) -> Result<Transfer, Error> {
    if let Some(&fence) = self.buffer_used.get(&buffer_id) {
      unsafe {
        self.device.wait_for_fences(&[fence], true, u64::MAX)?;
      }
    }

    let mut cmd_index = None;
    while cmd_index.is_none() {
      for i in 0..self.command_buffers.len() {
        if unsafe { self.device.get_fence_status(self.fences[i])? } {
          cmd_index = Some(i);
          break;
        }
      }
    }
    let index = cmd_index.unwrap();
    let command_buffer = self.command_buffers[index];
    let fence = self.fences[index];

    if let Some((&done, _)) = self.buffer_used.iter().find(|(_, &f)| fence == f) {
      self.buffer_used.remove(&done);
    }

    Ok(Transfer {
      buffer: command_buffer,
      fence,
    })
  }

  pub fn cleanup(&mut self) -> Result<(), Error> {
    for &fence in &self.fences {
      unsafe {
        self.device.destroy_fence(fence, None);
      }
    }
    for (_, buffer) in std::mem::take(&mut self.buffers) {
      buffer.cleanup(&self.device, &mut self.allocator)?;
    }
    for (_, image) in std::mem::take(&mut self.images) {
      image.cleanup(&self.device, &mut self.allocator)?;
    }
    unsafe {
      ManuallyDrop::drop(&mut self.allocator);
    }
    Ok(())
  }
}

impl From<BufferBlockSize> for usize {
  fn from(value: BufferBlockSize) -> Self {
    match value {
      BufferBlockSize::Large => BUFFER_BLOCK_SIZE_LARGE,
      BufferBlockSize::Medium => BUFFER_BLOCK_SIZE_MEDIUM,
      BufferBlockSize::Small => BUFFER_BLOCK_SIZE_SMALL,
      BufferBlockSize::Exact(size) => size,
    }
  }
}

pub struct Transfer {
  buffer: vk::CommandBuffer,
  fence: vk::Fence,
}

impl Transfer {
  pub fn buffer(&self) -> vk::CommandBuffer {
    self.buffer
  }

  pub fn fence(&self) -> vk::Fence {
    self.fence
  }

  pub fn wait(&self, device: &ash::Device) -> Result<(), vk::Result> {
    unsafe { device.wait_for_fences(&[self.fence], true, u64::MAX) }
  }

  pub fn reset(&self, device: &ash::Device) -> Result<(), vk::Result> {
    unsafe { device.reset_fences(&[self.fence]) }
  }
}
