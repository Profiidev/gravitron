use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use super::manager::Transfer;

pub struct Buffer {
  buffer: vk::Buffer,
  allocation: vulkan::Allocation,
  size: usize,
  usage: vk::BufferUsageFlags,
  location: gpu_allocator::MemoryLocation,
}

impl Buffer {
  pub fn new(
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    size: usize,
    usage: vk::BufferUsageFlags,
    location: gpu_allocator::MemoryLocation,
  ) -> Result<Self, Error> {
    let (buffer, allocation) = create_buffer(size, usage, location, device, allocator)?;

    Ok(Self {
      buffer,
      allocation,
      size,
      usage,
      location,
    })
  }

  pub fn write(
    &mut self,
    src_ptr: *const u8,
    size: usize,
    offset: usize,
  ) -> Result<(), vk::Result> {
    if size + offset > self.size {
      return Err(vk::Result::ERROR_OUT_OF_HOST_MEMORY);
    }
    let data_ptr = unsafe {
      self
        .allocation
        .mapped_ptr()
        .ok_or(vk::Result::ERROR_OUT_OF_HOST_MEMORY)?
        .byte_add(offset)
    }
    .as_ptr() as *mut u8;

    unsafe {
      data_ptr.copy_from_nonoverlapping(src_ptr, size);
    }
    Ok(())
  }

  pub fn fill<T: Sized>(&mut self, data: &[T]) -> Result<(), vk::Result> {
    let ptr = data.as_ptr() as *const u8;
    let count = std::mem::size_of_val(data);
    self.write(ptr, count, 0)
  }

  pub fn resize(
    &mut self,
    size: usize,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    if self.size == size {
      return Ok(());
    }

    let (buffer, allocation) = create_buffer(size, self.usage, self.location, device, allocator)?;

    let old_buffer = std::mem::replace(&mut self.buffer, buffer);
    let old_allocation = std::mem::replace(&mut self.allocation, allocation);
    self.size = size;

    unsafe {
      device.destroy_buffer(old_buffer, None);
      allocator.free(old_allocation)?;
    }

    Ok(())
  }

  pub unsafe fn cleanup(
    self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    device.destroy_buffer(self.buffer, None);
    allocator.free(self.allocation)?;
    Ok(())
  }

  pub fn buffer(&self) -> vk::Buffer {
    self.buffer
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn usage(&self) -> vk::BufferUsageFlags {
    self.usage
  }

  pub fn location(&self) -> gpu_allocator::MemoryLocation {
    self.location
  }

  pub unsafe fn ptr(&self) -> Option<*const u8> {
    Some(self.allocation.mapped_ptr()?.as_ptr() as *const u8)
  }
}

fn create_buffer(
  size: usize,
  usage: vk::BufferUsageFlags,
  location: gpu_allocator::MemoryLocation,
  device: &ash::Device,
  allocator: &mut vulkan::Allocator,
) -> Result<(vk::Buffer, vulkan::Allocation), Error> {
  let buffer_create_info = vk::BufferCreateInfo::default()
    .size(size as u64)
    .usage(usage);
  let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };

  let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
  let allocation_create_desc = vulkan::AllocationCreateDesc {
    requirements,
    location,
    linear: true,
    allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
    name: "Buffer",
  };
  let allocation = allocator.allocate(&allocation_create_desc)?;

  unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())? };

  Ok((buffer, allocation))
}

pub fn buffer_copy_info(dst_offset: usize, size: usize) -> Vec<vk::BufferCopy> {
  vec![vk::BufferCopy::default()
    .dst_offset(dst_offset as u64)
    .size(size as u64)]
}

pub fn buffer_copy(
  src: &Buffer,
  dst: &Buffer,
  device: &ash::Device,
  transfer_queue: vk::Queue,
  transfer: &Transfer,
  regions: &[vk::BufferCopy],
) -> Result<(), vk::Result> {
  let command_buffer = transfer.buffer();
  let begin_info = vk::CommandBufferBeginInfo::default();
  let buffers = [command_buffer];
  let submits = [vk::SubmitInfo::default().command_buffers(&buffers)];

  unsafe {
    device.begin_command_buffer(command_buffer, &begin_info)?;
    device.cmd_copy_buffer(command_buffer, src.buffer(), dst.buffer(), regions);
    device.end_command_buffer(command_buffer)?;
    device.queue_submit(transfer_queue, &submits, transfer.fence())
  }
}
