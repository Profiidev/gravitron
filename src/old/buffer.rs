use ash::vk;
use gpu_allocator::vulkan;

pub struct Buffer {
  pub buffer: vk::Buffer,
  pub allocation: vulkan::Allocation,
  pub size: u64,
}

impl Buffer {
  pub fn new(
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
    size: u64,
    usage: vk::BufferUsageFlags,
    memory_location: gpu_allocator::MemoryLocation,
  ) -> Result<Self, vk::Result> {
    let buffer_create_info = vk::BufferCreateInfo::default().size(size).usage(usage);
    let buffer = unsafe { device.create_buffer(&buffer_create_info, None) }?;
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation_create_desc = vulkan::AllocationCreateDesc {
      requirements,
      location: memory_location,
      linear: true,
      allocation_scheme: vulkan::AllocationScheme::GpuAllocatorManaged,
      name: "Buffer",
    };
    let allocation = allocator.allocate(&allocation_create_desc).unwrap();

    unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }?;

    Ok(Self {
      buffer,
      allocation,
      size,
    })
  }

  pub fn fill<T: Sized>(&mut self, data: &[T]) -> Result<(), vk::Result> {
    let bytes_to_write = std::mem::size_of_val(data) as u64;
    if bytes_to_write > self.size {
      return Err(vk::Result::ERROR_OUT_OF_HOST_MEMORY);
    }
    let data_ptr = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
    unsafe {
      data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
    }
    Ok(())
  }
}