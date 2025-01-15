use std::collections::HashMap;

use anyhow::Error;
use ash::vk;

use crate::{
  config::vulkan::{DescriptorSet, DescriptorType},
  vulkan::{
    graphics::swapchain::{SwapChain, IMAGES_PER_FRAME_BUFFER},
    memory::{manager::MemoryManager, types::BufferBlockSize, BufferMemory},
  },
};

#[allow(clippy::complexity)]
pub fn get_descriptor_set_layouts(
  descriptor_sets_config: &[DescriptorSet],
  descriptor_pool: vk::DescriptorPool,
  logical_device: &ash::Device,
  memory_manager: &mut MemoryManager,
) -> Result<
  (
    Vec<vk::DescriptorSetLayout>,
    Vec<vk::DescriptorSet>,
    HashMap<usize, HashMap<usize, BufferMemory>>,
  ),
  Error,
> {
  if descriptor_sets_config.is_empty() {
    return Ok((vec![], vec![], HashMap::new()));
  }

  let mut descriptor_layouts = vec![];

  for descriptor_set in descriptor_sets_config {
    let mut descriptor_set_layout_binding_descs = vec![];

    for (i, descriptor) in descriptor_set.descriptors.iter().enumerate() {
      match descriptor {
        DescriptorType::StorageBuffer(desc) | DescriptorType::UniformBuffer(desc) => {
          descriptor_set_layout_binding_descs.push(
            vk::DescriptorSetLayoutBinding::default()
              .binding(i as u32)
              .descriptor_type(desc.r#type)
              .descriptor_count(1)
              .stage_flags(desc.stage),
          );
        }
        DescriptorType::Image(desc) => {
          descriptor_set_layout_binding_descs.push(
            vk::DescriptorSetLayoutBinding::default()
              .binding(i as u32)
              .descriptor_type(desc.r#type)
              .descriptor_count(desc.images.len() as u32)
              .stage_flags(desc.stage),
          );
        }
      }
    }

    let descriptor_set_layout_create_info =
      vk::DescriptorSetLayoutCreateInfo::default().bindings(&descriptor_set_layout_binding_descs);
    let descriptor_set_layout = unsafe {
      logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
    }?;
    descriptor_layouts.push(descriptor_set_layout);
  }

  let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
    .descriptor_pool(descriptor_pool)
    .set_layouts(&descriptor_layouts);
  let descriptor_sets =
    unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info)? };

  let mut descriptor_buffers = HashMap::new();

  for (j, descriptor_set) in descriptor_sets_config.iter().enumerate() {
    let mut buffers = HashMap::new();

    for (i, descriptor) in descriptor_set.descriptors.iter().enumerate() {
      match descriptor {
        DescriptorType::StorageBuffer(desc) | DescriptorType::UniformBuffer(desc) => {
          let buffer = memory_manager
            .create_advanced_buffer(desc.buffer_usage, BufferBlockSize::Exact(desc.size))?;
          let mem = memory_manager
            .reserve_buffer_mem(buffer, desc.size)
            .unwrap()
            .0;

          buffers.insert(i, mem);

          let buffer_info_descriptor = [vk::DescriptorBufferInfo::default()
            .buffer(memory_manager.get_vk_buffer(buffer).unwrap())
            .offset(0)
            .range(desc.size as u64)];

          let write_desc_set = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_sets[j])
            .dst_binding(i as u32)
            .descriptor_type(desc.r#type)
            .buffer_info(&buffer_info_descriptor);

          unsafe {
            logical_device.update_descriptor_sets(&[write_desc_set], &[]);
          }
        }
        DescriptorType::Image(desc) => {
          if desc.images.is_empty() {
            continue;
          }

          let mut image_infos = Vec::new();
          for image in &desc.images {
            let sampler_image = memory_manager.create_texture_image(image)?;
            let view = memory_manager.get_vk_image_view(sampler_image).unwrap();
            let sampler = memory_manager.get_vk_sampler(sampler_image).unwrap();

            image_infos.push(
              vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(view)
                .sampler(sampler),
            );
          }

          let write_desc_set = vk::WriteDescriptorSet::default()
            .dst_binding(i as u32)
            .dst_set(descriptor_sets[j])
            .descriptor_type(desc.r#type)
            .image_info(&image_infos);

          unsafe {
            logical_device.update_descriptor_sets(&[write_desc_set], &[]);
          }
        }
      }
    }

    descriptor_buffers.insert(j, buffers);
  }

  Ok((descriptor_layouts, descriptor_sets, descriptor_buffers))
}

pub fn get_light_framebuffer_descriptor_set(
  logical_device: &ash::Device,
  descriptor_pool: vk::DescriptorPool,
  swapchain: &SwapChain,
) -> Result<(vk::DescriptorSetLayout, vk::DescriptorSet), Error> {
  let layout_binding_descs = (0..IMAGES_PER_FRAME_BUFFER)
    .map(|i| {
      vk::DescriptorSetLayoutBinding::default()
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .binding(i)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
    })
    .collect::<Vec<vk::DescriptorSetLayoutBinding>>();

  let layout_create_info =
    vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_binding_descs);
  let descriptor_set_layout =
    [unsafe { logical_device.create_descriptor_set_layout(&layout_create_info, None) }?];

  let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
    .descriptor_pool(descriptor_pool)
    .set_layouts(&descriptor_set_layout);
  let descriptor_sets =
    unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info)? };

  let image_infos = swapchain
    .framebuffer_image_handles()
    .iter()
    .map(|(view, sampler)| {
      vec![vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(*view)
        .sampler(*sampler)]
    })
    .collect::<Vec<Vec<vk::DescriptorImageInfo>>>();

  let write_desc_set = (0..IMAGES_PER_FRAME_BUFFER)
    .map(|i| {
      vk::WriteDescriptorSet::default()
        .dst_binding(i)
        .dst_set(descriptor_sets[0])
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&image_infos[i as usize])
    })
    .collect::<Vec<vk::WriteDescriptorSet>>();

  unsafe {
    logical_device.update_descriptor_sets(&write_desc_set, &[]);
  }

  Ok((descriptor_set_layout[0], descriptor_sets[0]))
}

pub fn add_descriptor(pool_sizes: &mut Vec<vk::DescriptorPoolSize>, desc: &DescriptorType) {
  match desc {
    DescriptorType::StorageBuffer(desc) | DescriptorType::UniformBuffer(desc) => {
      internal_add(pool_sizes, desc.r#type);
    }
    DescriptorType::Image(desc) => {
      internal_add(pool_sizes, desc.r#type);
    }
  }
  fn internal_add(pool_sizes: &mut Vec<vk::DescriptorPoolSize>, ty: vk::DescriptorType) {
    if let Some(pool) = pool_sizes.iter_mut().find(|s| s.ty == ty) {
      pool.descriptor_count += 1;
    } else {
      pool_sizes.push(vk::DescriptorPoolSize::default().ty(ty).descriptor_count(1));
    }
  }
}
