use std::collections::HashMap;

use crate::Id;
use anyhow::Error;
use ash::vk;
use gpu_allocator::vulkan;

use crate::vulkan::shader::buffer::Buffer;

pub struct ModelManager {
  models: Vec<Model>,
}

pub const CUBE_MODEL: Id = 0;

pub struct Model {
  index_len: u32,
  vertex_buffer: Option<Buffer>,
  index_buffer: Option<Buffer>,
  instance_buffer: Option<Buffer>,
}

#[derive(Debug)]
#[repr(C)]
pub struct VertexData {
  position: glam::Vec3,
  normal: glam::Vec3,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct InstanceData {
  model_matrix: glam::Mat4,
  inv_model_matrix: glam::Mat4,
  color: glam::Vec3,
  metallic: f32,
  roughness: f32,
}

impl ModelManager {
  pub fn new(device: &ash::Device, allocator: &mut vulkan::Allocator) -> Self {
    let models = vec![cube(device, allocator)];

    ModelManager { models }
  }

  pub fn add_model(
    &mut self,
    vertex_data: Vec<VertexData>,
    index_data: Vec<u32>,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<Id, Error> {
    self
      .models
      .push(Model::new(vertex_data, index_data, device, allocator)?);
    Ok(self.models.len() as Id - 1)
  }

  pub fn cleanup(&mut self, device: &ash::Device, allocator: &mut vulkan::Allocator) {
    for model in &mut self.models {
      model.cleanup(device, allocator).unwrap();
    }
  }

  pub fn update_instance_buffer(
    &mut self,
    instances: &HashMap<Id, Vec<InstanceData>>,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    for (i, model) in self.models.iter_mut().enumerate() {
      if let Some(instance) = instances.get(&(i as Id)) {
        model.update_instance_buffer(instance, device, allocator)?;
      }
    }
    Ok(())
  }

  pub fn record_command_buffer(
    &self,
    instances: &HashMap<Id, Vec<InstanceData>>,
    command_buffer: vk::CommandBuffer,
    device: &ash::Device,
  ) {
    for (i, model) in self.models.iter().enumerate() {
      if let Some(instance) = instances.get(&(i as Id)) {
        model.record_command_buffer(instance.len() as u32, command_buffer, device);
      }
    }
  }
}

impl Model {
  fn new(
    vertex_data: Vec<VertexData>,
    index_data: Vec<u32>,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<Self, Error> {
    let vertex_data_slice = vertex_data.as_slice();
    let mut vertex_buffer = Buffer::new(
      allocator,
      device,
      std::mem::size_of_val(vertex_data_slice) as u64,
      vk::BufferUsageFlags::VERTEX_BUFFER,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;
    vertex_buffer.fill(vertex_data_slice)?;

    let index_data_slice = index_data.as_slice();
    let mut index_buffer = Buffer::new(
      allocator,
      device,
      std::mem::size_of_val(index_data_slice) as u64,
      vk::BufferUsageFlags::INDEX_BUFFER,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;
    index_buffer.fill(index_data_slice)?;

    let instance_buffer = Buffer::new(
      allocator,
      device,
      std::mem::size_of::<InstanceData>() as u64 * 2,
      vk::BufferUsageFlags::VERTEX_BUFFER,
      gpu_allocator::MemoryLocation::CpuToGpu,
    )?;

    Ok(Self {
      index_len: index_data.len() as u32,
      vertex_buffer: Some(vertex_buffer),
      index_buffer: Some(index_buffer),
      instance_buffer: Some(instance_buffer),
    })
  }

  fn cleanup(
    &mut self,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    if let Some(buffer) = self.vertex_buffer.take() {
      unsafe {
        buffer.cleanup(device, allocator)?;
      }
    }
    if let Some(buffer) = self.index_buffer.take() {
      unsafe {
        buffer.cleanup(device, allocator)?;
      }
    }
    if let Some(buffer) = self.instance_buffer.take() {
      unsafe {
        buffer.cleanup(device, allocator)?;
      }
    }
    Ok(())
  }

  fn update_instance_buffer(
    &mut self,
    instances: &[InstanceData],
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
  ) -> Result<(), Error> {
    self.instance_buffer.as_mut().unwrap().resize(
      std::mem::size_of_val(instances) as u64,
      device,
      allocator,
    )?;
    Ok(self.instance_buffer.as_mut().unwrap().fill(instances)?)
  }

  pub fn record_command_buffer(
    &self,
    instance_count: u32,
    command_buffer: vk::CommandBuffer,
    device: &ash::Device,
  ) {
    unsafe {
      device.cmd_bind_vertex_buffers(
        command_buffer,
        0,
        &[self.vertex_buffer.as_ref().unwrap().buffer()],
        &[0],
      );
      device.cmd_bind_index_buffer(
        command_buffer,
        self.index_buffer.as_ref().unwrap().buffer(),
        0,
        vk::IndexType::UINT32,
      );
      device.cmd_bind_vertex_buffers(
        command_buffer,
        1,
        &[self.instance_buffer.as_ref().unwrap().buffer()],
        &[0],
      );
      device.cmd_draw_indexed(command_buffer, self.index_len, instance_count, 0, 0, 0);
    }
  }
}

fn cube(device: &ash::Device, allocator: &mut vulkan::Allocator) -> Model {
  let lbf = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
  };
  let lbb = VertexData {
    position: glam::Vec3::new(-1.0, 1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
  };
  let ltf = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
  };
  let ltb = VertexData {
    position: glam::Vec3::new(-1.0, -1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
  };
  let rbf = VertexData {
    position: glam::Vec3::new(1.0, 1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
  };
  let rbb = VertexData {
    position: glam::Vec3::new(1.0, 1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
  };
  let rtf = VertexData {
    position: glam::Vec3::new(1.0, -1.0, -1.0),
    normal: glam::Vec3::new(0.0, 0.0, -1.0),
  };
  let rtb = VertexData {
    position: glam::Vec3::new(1.0, -1.0, 1.0),
    normal: glam::Vec3::new(0.0, 0.0, 1.0),
  };

  Model::new(
    vec![lbf, lbb, ltf, ltb, rbf, rbb, rtf, rtb],
    vec![
      0, 1, 5, 0, 5, 4, //bottom
      2, 7, 3, 2, 6, 7, //top
      0, 6, 2, 0, 4, 6, //front
      1, 3, 7, 1, 7, 5, //back
      0, 2, 1, 1, 2, 3, //left
      4, 5, 6, 5, 7, 6, //right
    ],
    device,
    allocator,
  )
  .unwrap()
}

impl InstanceData {
  pub fn new(
    model_matrix: glam::Mat4,
    inv_model_matrix: glam::Mat4,
    color: glam::Vec3,
    metallic: f32,
    roughness: f32,
  ) -> Self {
    Self {
      model_matrix,
      inv_model_matrix,
      color,
      metallic,
      roughness,
    }
  }
}
