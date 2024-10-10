use std::{collections::HashMap, mem::ManuallyDrop, u64};

use anyhow::Error;
use ash::vk::{self, ImageViewCreateInfo};
use gpu_allocator::vulkan;

use crate::vulkan::{
  device::Device,
  instance::InstanceDevice,
  pipeline::pools::{CommandBufferType, Pools},
};

use super::{
  allocator::{Allocator, BufferMemory},
  buffer::Buffer,
  image::Image,
};

pub type BufferId = crate::Id;
pub type ImageId = crate::Id;

pub struct MemoryManager {
  buffers: HashMap<BufferId, ManagedBuffer>,
  buffer_used: HashMap<BufferId, vk::Fence>,
  last_buffer_id: BufferId,
  images: HashMap<ImageId, Image>,
  last_image_id: BufferId,
  allocator: ManuallyDrop<vulkan::Allocator>,
  device: ash::Device,
  buffer_size: usize,
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
      buffer_size: 1024 * 1024 * 64,
      command_buffers,
      fences,
      transfer_queue: device.get_queues().transfer(),
    })
  }

  pub fn create_buffer(&mut self, usage: vk::BufferUsageFlags) -> Result<BufferId, Error> {
    let buffer = ManagedBuffer::new(&mut self.allocator, &self.device, self.buffer_size, usage)?;

    self.buffers.insert(self.last_buffer_id, buffer);
    let id = self.last_buffer_id;
    self.last_buffer_id += 1;
    Ok(id)
  }

  pub fn create_image(&mut self, location: gpu_allocator::MemoryLocation, image_info: &vk::ImageCreateInfo, image_view_info: &ImageViewCreateInfo) -> Result<ImageId, Error> {
    let image = Image::new(&self.device, &mut self.allocator, location, image_info, image_view_info)?;

    self.images.insert(self.last_image_id, image);
    let id = self.last_image_id;
    self.last_image_id += 1;
    Ok(id)
  }

  pub fn add_to_buffer<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
  ) -> Option<BufferMemory> {
    let (command_buffer, fence, size) = self.write_prepare_internal(buffer_id, data)?;
    let buffer = self.buffers.get_mut(&buffer_id)?;

    let mem = if let Some(mem) = buffer.allocator.alloc(size) {
      mem
    } else {
      let additional_size =
        (size as f32 / self.buffer_size as f32).ceil() as usize * self.buffer_size;
      let new_gpu = Buffer::new(
        &mut self.allocator,
        &self.device,
        buffer.gpu.size() + additional_size,
        buffer.gpu.usage(),
        gpu_allocator::MemoryLocation::GpuOnly,
      )
      .ok()?;
      buffer_copy(
        &buffer.gpu,
        &new_gpu,
        &self.device,
        self.transfer_queue,
        command_buffer,
        fence,
        0,
        buffer.gpu.size(),
      );

      unsafe {
        self.device.wait_for_fences(&[fence], true, u64::MAX).ok()?;
        self.device.reset_fences(&[fence]).ok()?;
      }

      let old_gpu = std::mem::replace(&mut buffer.gpu, new_gpu);
      unsafe { old_gpu.cleanup(&self.device, &mut self.allocator) };

      buffer.allocator.grow(additional_size);

      buffer.allocator.alloc(size).unwrap()
    };

    buffer_copy(
      &buffer.transfer,
      &buffer.gpu,
      &self.device,
      self.transfer_queue,
      command_buffer,
      fence,
      mem.offset(),
      size,
    )
    .ok()?;

    self.buffer_used.insert(buffer_id, fence);

    Some(mem)
  }

  pub fn write_to_buffer<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    mem: &BufferMemory,
    data: &[T],
  ) -> Option<()> {
    let (command_buffer, fence, size) = self.write_prepare_internal(buffer_id, data)?;
    let buffer = self.buffers.get_mut(&buffer_id)?;

    buffer_copy(
      &buffer.transfer,
      &buffer.gpu,
      &self.device,
      self.transfer_queue,
      command_buffer,
      fence,
      mem.offset(),
      size,
    )
    .ok()?;

    self.buffer_used.insert(buffer_id, fence);

    Some(())
  }

  fn write_prepare_internal<T: Sized>(
    &mut self,
    buffer_id: BufferId,
    data: &[T],
  ) -> Option<(vk::CommandBuffer, vk::Fence, usize)> {
    let buffer = self.buffers.get_mut(&buffer_id)?;
    let size = std::mem::size_of_val(&data);

    if let Some(&fence) = self.buffer_used.get(&buffer_id) {
      unsafe {
        self.device.wait_for_fences(&[fence], true, u64::MAX).ok()?;
      }
    }

    if buffer.transfer.size() < size {
      buffer
        .transfer
        .resize(size, &self.device, &mut self.allocator)
        .ok();
    }
    buffer.transfer.fill(data);

    let mut cmd_index = None;
    while cmd_index.is_none() {
      for i in 0..self.command_buffers.len() {
        if unsafe { self.device.get_fence_status(self.fences[i]).ok()? } {
          cmd_index = Some(i);
          unsafe { self.device.reset_fences(&[self.fences[i]]) };
          break;
        }
      }
    }
    let index = cmd_index.unwrap();
    let command_buffer = self.command_buffers[index];
    let fence = self.fences[index];

    Some((command_buffer, fence, size))
  }

  pub fn free(&mut self, buffer_id: BufferId, mem: BufferMemory) {
    let buffer = self.buffers.get_mut(&buffer_id).unwrap();
    buffer.allocator.free(mem);
  }

  pub fn get_vk_buffer(&self, buffer_id: BufferId) -> Option<vk::Buffer> {
    Some(self.buffers.get(&buffer_id)?.gpu.buffer())
  }

  pub fn get_vk_image_view(&self, image_id: ImageId) -> Option<vk::ImageView> {
    Some(self.images.get(&image_id)?.image_view())
  }

  pub fn cleanup(&mut self) -> Result<(), Error> {
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

struct ManagedBuffer {
  transfer: Buffer,
  gpu: Buffer,
  allocator: Allocator,
}

impl ManagedBuffer {
  fn new(
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    size: usize,
    usage: vk::BufferUsageFlags,
  ) -> Result<Self, Error> {
    let transfer = Buffer::new(
      allocator,
      device,
      size,
      usage | vk::BufferUsageFlags::TRANSFER_SRC,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;

    let gpu = Buffer::new(
      allocator,
      device,
      size,
      usage | vk::BufferUsageFlags::TRANSFER_DST,
      gpu_allocator::MemoryLocation::GpuOnly,
    )?;

    let allocator = Allocator::new(size);

    Ok(Self {
      transfer,
      gpu,
      allocator,
    })
  }

  fn cleanup(self, device: &ash::Device, allocator: &mut vulkan::Allocator) -> Result<(), Error> {
    unsafe {
      self.transfer.cleanup(device, allocator)?;
      self.gpu.cleanup(device, allocator)
    }
  }
}

fn buffer_copy(
  src: &Buffer,
  dst: &Buffer,
  device: &ash::Device,
  transfer_queue: vk::Queue,
  command_buffer: vk::CommandBuffer,
  fence: vk::Fence,
  dst_offset: usize,
  size: usize,
) -> Result<(), vk::Result> {
  let begin_info = vk::CommandBufferBeginInfo::default();
  let regions = [vk::BufferCopy::default()
    .dst_offset(dst_offset as u64)
    .size(size as u64)];
  let buffers = [command_buffer];
  let submits = [vk::SubmitInfo::default().command_buffers(&buffers)];
  unsafe {
    device.begin_command_buffer(command_buffer, &begin_info)?;
    device.cmd_copy_buffer(command_buffer, src.buffer(), dst.buffer(), &regions);
    device.end_command_buffer(command_buffer)?;
    device.queue_submit(transfer_queue, &submits, fence)
  }
}
