use std::{collections::HashMap, mem::ManuallyDrop, path::Path};

use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use crate::vulkan::{
  device::Device,
  instance::InstanceDevice,
  pipeline::pools::{CommandBufferType, Pools},
};
use crate::Id;

use super::{
  advanced_buffer::AdvancedBuffer,
  allocator::BufferMemory,
  image::Image,
  simple_buffer::SimpleBuffer,
  texture::Texture,
  types::{
    BufferBlockSize, BufferId, BufferType, ImageId, ImageType, BUFFER_BLOCK_SIZE_LARGE,
    BUFFER_BLOCK_SIZE_MEDIUM, BUFFER_BLOCK_SIZE_SMALL,
  },
};

pub struct MemoryManager {
  buffers: HashMap<BufferId, BufferType>,
  buffer_used: HashMap<BufferId, vk::Fence>,
  last_buffer_id: Id,
  images: HashMap<ImageId, ImageType>,
  last_image_id: Id,
  allocator: ManuallyDrop<vulkan::Allocator>,
  device: ash::Device,
  command_buffers: Vec<vk::CommandBuffer>,
  fences: Vec<vk::Fence>,
  transfer_queue: vk::Queue,
  graphics_queue: vk::Queue,
  graphics_buffer: (vk::CommandBuffer, vk::Fence),
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

    let graphics_buffer =
      pools.create_command_buffers(logical_device, 1, CommandBufferType::Graphics)?[0];
    let fence = unsafe { logical_device.create_fence(&fence_create_info, None)? };

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
      graphics_queue: device.get_queues().graphics(),
      graphics_buffer: (graphics_buffer, fence),
    })
  }

  pub fn create_advanced_buffer(
    &mut self,
    usage: vk::BufferUsageFlags,
    block_size: BufferBlockSize,
  ) -> Result<BufferId, Error> {
    let id = BufferId::Advanced(self.last_buffer_id);
    let buffer = AdvancedBuffer::new(
      id,
      &mut self.allocator,
      &self.device,
      usage,
      block_size.into(),
    )?;

    self.buffers.insert(id, BufferType::Advanced(buffer));
    self.last_buffer_id += 1;
    Ok(id)
  }

  pub fn create_simple_buffer(
    &mut self,
    usage: vk::BufferUsageFlags,
    block_size: BufferBlockSize,
  ) -> Result<BufferId, Error> {
    let id = BufferId::Simple(self.last_buffer_id);
    let buffer = SimpleBuffer::new(
      id,
      &mut self.allocator,
      &self.device,
      usage,
      block_size.into(),
    )?;

    self.buffers.insert(id, BufferType::Simple(buffer));
    self.last_buffer_id += 1;
    Ok(id)
  }

  pub fn create_image(
    &mut self,
    location: gpu_allocator::MemoryLocation,
    image_info: &vk::ImageCreateInfo,
    image_view_info: &vk::ImageViewCreateInfo,
  ) -> Result<ImageId, Error> {
    let id = ImageId::Simple(self.last_image_id);

    let image = Image::new(
      &self.device,
      &mut self.allocator,
      location,
      image_info,
      image_view_info,
    )?;

    self.images.insert(id, ImageType::Simple(image));

    self.last_image_id += 1;
    Ok(id)
  }

  pub fn create_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<ImageId, Error> {
    let id = ImageId::Texture(self.last_image_id);
    let transfer = Transfer {
      buffer: self.graphics_buffer.0,
      fence: self.graphics_buffer.1,
      queue: self.graphics_queue,
    };

    let texture = Texture::new(path, &self.device, &mut self.allocator, &transfer)?;

    self.images.insert(id, ImageType::Texture(texture));

    self.last_image_id += 1;
    Ok(id)
  }

  pub fn reserve_buffer_mem(
    &mut self,
    buffer_id: BufferId,
    size: usize,
  ) -> Option<(BufferMemory, bool)> {
    let transfer = self.reserve_transfer(buffer_id).ok()?;

    match self.buffers.get_mut(&buffer_id)? {
      BufferType::Advanced(buffer) => {
        buffer.reserve_buffer_mem(size, &self.device, &mut self.allocator, &transfer)
      }
      BufferType::Simple(buffer) => {
        buffer.reserve_buffer_mem(size, &self.device, &mut self.allocator)
      }
    }
  }

  pub fn add_to_buffer<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
  ) -> Option<(BufferMemory, bool)> {
    let transfer = self.reserve_transfer(buffer_id).ok()?;

    match self.buffers.get_mut(&buffer_id)? {
      BufferType::Advanced(buffer) => {
        let mem = buffer.add_to_buffer(data, &self.device, &mut self.allocator, &transfer);

        self.buffer_used.insert(buffer_id, transfer.fence);
        mem
      }
      BufferType::Simple(buffer) => buffer.add_to_buffer(data, &self.device, &mut self.allocator),
    }
  }

  pub fn write_to_buffer<T: Sized>(&mut self, mem: &BufferMemory, data: &[T]) -> Option<()> {
    let id = mem.buffer();
    let transfer = self.reserve_transfer(id).ok()?;

    match self.buffers.get_mut(&mem.buffer())? {
      BufferType::Advanced(buffer) => {
        buffer.write_to_buffer(mem, data, &self.device, &mut self.allocator, &transfer)?;

        self.buffer_used.insert(id, transfer.fence);
        Some(())
      }
      BufferType::Simple(buffer) => buffer.write_to_buffer(mem, data),
    }
  }

  pub fn write_to_buffer_direct<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
    regions: &[vk::BufferCopy],
  ) -> Option<()> {
    let transfer = self.reserve_transfer(buffer_id).ok()?;

    match self.buffers.get_mut(&buffer_id)? {
      BufferType::Advanced(buffer) => {
        buffer.write_to_buffer_direct(
          data,
          regions,
          &self.device,
          &mut self.allocator,
          &transfer,
        )?;

        self.buffer_used.insert(buffer_id, transfer.fence);
        Some(())
      }
      BufferType::Simple(buffer) => buffer.write_to_buffer_direct(data, regions),
    }
  }

  pub fn resize_buffer_mem(&mut self, mem: &mut BufferMemory, size: usize) -> Option<bool> {
    let transfer = self.reserve_transfer(mem.buffer()).ok()?;

    match self.buffers.get_mut(&mem.buffer())? {
      BufferType::Advanced(buffer) => {
        buffer.resize_buffer_mem(mem, size, &self.device, &mut self.allocator, &transfer)
      }
      BufferType::Simple(buffer) => {
        buffer.resize_buffer_mem(mem, size, &self.device, &mut self.allocator)
      }
    }
  }

  pub fn free_buffer_mem(&mut self, mem: BufferMemory) {
    match self.buffers.get_mut(&mem.buffer()).unwrap() {
      BufferType::Advanced(buffer) => {
        buffer.free_buffer_mem(mem);
      }
      BufferType::Simple(buffer) => {
        buffer.free_buffer_mem(mem);
      }
    }
  }

  pub fn get_vk_buffer(&self, buffer_id: BufferId) -> Option<vk::Buffer> {
    match self.buffers.get(&buffer_id)? {
      BufferType::Advanced(buffer) => Some(buffer.vk_buffer()),
      BufferType::Simple(buffer) => Some(buffer.vk_buffer()),
    }
  }

  pub fn get_vk_image_view(&self, image_id: ImageId) -> Option<vk::ImageView> {
    match self.images.get(&image_id)? {
      ImageType::Simple(image) => Some(image.image_view()),
      ImageType::Texture(image) => Some(image.image_view()),
    }
  }

  pub fn get_vk_sampler(&self, image_id: ImageId) -> Option<vk::Sampler> {
    match self.images.get(&image_id)? {
      ImageType::Texture(id) => Some(id.sampler()),
      _ => None,
    }
  }

  pub fn get_buffer_size(&self, buffer_id: BufferId) -> Option<usize> {
    match self.buffers.get(&buffer_id)? {
      BufferType::Advanced(buffer) => Some(buffer.size()),
      BufferType::Simple(buffer) => Some(buffer.size()),
    }
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
      queue: self.transfer_queue,
    })
  }

  pub fn cleanup(&mut self) -> Result<(), Error> {
    for &fence in &self.fences {
      unsafe {
        self.device.destroy_fence(fence, None);
      }
    }
    unsafe {
      self.device.destroy_fence(self.graphics_buffer.1, None);
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
  queue: vk::Queue,
}

impl Transfer {
  pub fn buffer(&self) -> vk::CommandBuffer {
    self.buffer
  }

  pub fn fence(&self) -> vk::Fence {
    self.fence
  }

  pub fn queue(&self) -> vk::Queue {
    self.queue
  }

  pub fn wait(&self, device: &ash::Device) -> Result<(), vk::Result> {
    unsafe { device.wait_for_fences(&[self.fence], true, u64::MAX) }
  }

  pub fn reset(&self, device: &ash::Device) -> Result<(), vk::Result> {
    unsafe { device.reset_fences(&[self.fence]) }
  }
}
