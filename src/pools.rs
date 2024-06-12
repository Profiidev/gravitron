use ash::vk;

use crate::queues::QueueFamilies;

pub struct Pools {
  pub command_pool_graphics: vk::CommandPool,
  pub command_pool_transfer: vk::CommandPool,
}

impl Pools {
  pub fn init(
    logical_device: &ash::Device,
    queue_families: &QueueFamilies,
  ) -> Result<Self, vk::Result> {
    let command_pool_create_info = vk::CommandPoolCreateInfo::default()
      .queue_family_index(queue_families.graphics_q_index.unwrap())
      .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let command_pool_graphics =
      unsafe { logical_device.create_command_pool(&command_pool_create_info, None) }?;

    let command_pool_create_info = vk::CommandPoolCreateInfo::default()
      .queue_family_index(queue_families.transfer_q_index.unwrap())
      .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let command_pool_transfer =
      unsafe { logical_device.create_command_pool(&command_pool_create_info, None) }?;

    Ok(Self {
      command_pool_graphics,
      command_pool_transfer,
    })
  }

  pub unsafe fn cleanup(&self, logical_device: &ash::Device) {
    logical_device.destroy_command_pool(self.command_pool_graphics, None);
    logical_device.destroy_command_pool(self.command_pool_transfer, None);
  }
}

pub fn create_command_buffers(
  logical_device: &ash::Device,
  pools: &Pools,
  amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
  let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
    .command_pool(pools.command_pool_graphics)
    .command_buffer_count(amount as u32);
  unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
}