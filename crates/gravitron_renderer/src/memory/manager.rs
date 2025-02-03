use std::{collections::HashMap, mem::ManuallyDrop};

use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;
use gravitron_plugin::config::vulkan::ImageConfig;

use crate::{
  device::Device,
  instance::InstanceDevice,
  pipeline::pools::{CommandBufferType, Pools},
};

use super::{
  advanced_buffer::AdvancedBuffer,
  allocator::BufferMemory,
  error::MemoryError,
  image::Image,
  sampler_image::SamplerImage,
  simple_buffer::SimpleBuffer,
  types::{
    BufferBlockSize, BufferId, BufferMemoryLocation, BufferType, ImageId, ImageType,
    BUFFER_BLOCK_SIZE_LARGE, BUFFER_BLOCK_SIZE_MEDIUM, BUFFER_BLOCK_SIZE_SMALL,
  },
};

pub struct MemoryManager {
  buffers: HashMap<BufferId, BufferType>,
  buffer_used: HashMap<BufferId, vk::Fence>,
  last_buffer_id: u64,
  images: HashMap<ImageId, ImageType>,
  last_image_id: u64,
  allocator: ManuallyDrop<vulkan::Allocator>,
  device: ash::Device,
  transfers: Vec<Transfer>,
  graphics_transfer: Transfer,
}

impl MemoryManager {
  pub(crate) fn new(
    instance: &InstanceDevice,
    device: &Device,
    pools: &mut Pools,
  ) -> Result<Self, Error> {
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

    let mut transfers = Vec::new();
    let queue = device.get_queues().transfer();

    for buffer in command_buffers {
      transfers.push(Transfer {
        buffer,
        fence: unsafe { logical_device.create_fence(&fence_create_info, None)? },
        queue,
      });
    }

    let graphics_buffer =
      pools.create_command_buffers(logical_device, 1, CommandBufferType::Graphics)?[0];
    let fence = unsafe { logical_device.create_fence(&fence_create_info, None)? };

    let graphics_transfer = Transfer {
      buffer: graphics_buffer,
      fence,
      queue: device.get_queues().graphics(),
    };

    Ok(Self {
      buffers: HashMap::new(),
      buffer_used: HashMap::new(),
      last_buffer_id: 0,
      images: HashMap::new(),
      last_image_id: 0,
      allocator: ManuallyDrop::new(allocator),
      device: logical_device.clone(),
      transfers,
      graphics_transfer,
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
    location: BufferMemoryLocation,
  ) -> Result<BufferId, Error> {
    let id = BufferId::Simple(self.last_buffer_id);
    let buffer = SimpleBuffer::new(
      id,
      &mut self.allocator,
      &self.device,
      usage,
      block_size.into(),
      location,
    )?;

    self.buffers.insert(id, BufferType::Simple(buffer));
    self.last_buffer_id += 1;
    Ok(id)
  }

  pub fn create_image(
    &mut self,
    image_info: &vk::ImageCreateInfo,
    image_view_info: &vk::ImageViewCreateInfo,
  ) -> Result<ImageId, Error> {
    let id = ImageId::Simple(self.last_image_id);

    let image = Image::new(
      &self.device,
      &mut self.allocator,
      image_info,
      image_view_info,
    )?;

    self.images.insert(id, ImageType::Simple(image));

    self.last_image_id += 1;
    Ok(id)
  }

  pub fn create_texture_image(&mut self, image_config: &ImageConfig) -> Result<ImageId, Error> {
    let id = ImageId::Sampler(self.last_image_id);

    let sampler_image = SamplerImage::new_texture(
      image_config,
      &self.device,
      &mut self.allocator,
      &self.graphics_transfer,
    )?;

    self.images.insert(id, ImageType::Sampler(sampler_image));

    self.last_image_id += 1;
    Ok(id)
  }

  pub fn create_sampler_image(
    &mut self,
    image_info: &vk::ImageCreateInfo,
    image_view_info: &vk::ImageViewCreateInfo,
    sampler_info: &vk::SamplerCreateInfo,
  ) -> Result<ImageId, Error> {
    let id = ImageId::Sampler(self.last_image_id);

    let sampler_image = SamplerImage::new(
      &self.device,
      &mut self.allocator,
      image_info,
      image_view_info,
      sampler_info,
    )?;

    self.images.insert(id, ImageType::Sampler(sampler_image));

    self.last_image_id += 1;
    Ok(id)
  }

  pub fn reserve_buffer_mem(&mut self, buffer_id: BufferId, size: usize) -> Option<BufferMemory> {
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
  ) -> Result<BufferMemory, Error> {
    let transfer = self.reserve_transfer(buffer_id)?;

    match self
      .buffers
      .get_mut(&buffer_id)
      .ok_or_else(|| MemoryError::NotFound)?
    {
      BufferType::Advanced(buffer) => {
        let mem = buffer.add_to_buffer(data, &self.device, &mut self.allocator, &transfer);

        self.buffer_used.insert(buffer_id, transfer.fence);
        mem
      }
      BufferType::Simple(buffer) => buffer.add_to_buffer(data, &self.device, &mut self.allocator),
    }
  }

  pub fn write_to_buffer<T: Sized>(&mut self, mem: &BufferMemory, data: &[T]) -> Result<(), Error> {
    let id = mem.buffer();
    let transfer = self.reserve_transfer(id)?;

    match self
      .buffers
      .get_mut(&mem.buffer())
      .ok_or_else(|| MemoryError::NotFound)?
    {
      BufferType::Advanced(buffer) => {
        buffer.write_to_buffer(mem, data, &self.device, &mut self.allocator, &transfer)?;

        self.buffer_used.insert(id, transfer.fence);
        Ok(())
      }
      BufferType::Simple(buffer) => buffer.write_to_buffer(mem, data),
    }
  }

  pub fn write_to_buffer_direct<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
    regions: &[vk::BufferCopy],
  ) -> Result<(), Error> {
    let transfer = self.reserve_transfer(buffer_id)?;

    match self
      .buffers
      .get_mut(&buffer_id)
      .ok_or_else(|| MemoryError::NotFound)?
    {
      BufferType::Advanced(buffer) => {
        buffer.write_to_buffer_direct(
          data,
          regions,
          &self.device,
          &mut self.allocator,
          &transfer,
        )?;

        self.buffer_used.insert(buffer_id, transfer.fence);
        Ok(())
      }
      BufferType::Simple(buffer) => buffer.write_to_buffer_direct(data, regions),
    }
  }

  pub fn resize_buffer_mem(&mut self, mem: &mut BufferMemory, size: usize) -> Result<(), Error> {
    let transfer = self.reserve_transfer(mem.buffer())?;

    match self
      .buffers
      .get_mut(&mem.buffer())
      .ok_or_else(|| MemoryError::NotFound)?
    {
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

  pub(crate) fn get_vk_buffer(&self, buffer_id: BufferId) -> Option<vk::Buffer> {
    match self.buffers.get(&buffer_id)? {
      BufferType::Advanced(buffer) => Some(buffer.vk_buffer()),
      BufferType::Simple(buffer) => Some(buffer.vk_buffer()),
    }
  }

  pub(crate) fn get_vk_image_view(&self, image_id: ImageId) -> Option<vk::ImageView> {
    match self.images.get(&image_id)? {
      ImageType::Simple(image) => Some(image.image_view()),
      ImageType::Sampler(image) => Some(image.image_view()),
    }
  }

  pub(crate) fn get_vk_sampler(&self, image_id: ImageId) -> Option<vk::Sampler> {
    match self.images.get(&image_id)? {
      ImageType::Sampler(id) => Some(id.sampler()),
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

    let mut transfer_found = None;
    while transfer_found.is_none() {
      for transfer in &self.transfers {
        if unsafe { self.device.get_fence_status(transfer.fence())? } {
          transfer_found = Some(transfer.clone());
          break;
        }
      }
    }
    let transfer = transfer_found.unwrap();

    if let Some((&done, _)) = self
      .buffer_used
      .iter()
      .find(|(_, &f)| transfer.fence() == f)
    {
      self.buffer_used.remove(&done);
    }

    Ok(transfer)
  }

  pub(crate) fn cleanup(&mut self) -> Result<(), Error> {
    for transfer in &self.transfers {
      unsafe {
        self.device.destroy_fence(transfer.fence(), None);
      }
    }
    unsafe {
      self
        .device
        .destroy_fence(self.graphics_transfer.fence(), None);
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

#[derive(Clone)]
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
