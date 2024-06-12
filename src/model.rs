use ash::vk;
use gpu_allocator::vulkan;

use crate::buffer::Buffer;

pub struct Model<V, I> {
  pub vertex_data: Vec<V>,
  pub handle_to_index: std::collections::HashMap<usize, usize>,
  pub handles: Vec<usize>,
  pub instances: Vec<I>,
  pub first_invisible: usize,
  pub next_handle: usize,
  pub vertex_buffer: Option<Buffer>,
  pub instance_buffer: Option<Buffer>,
}

#[repr(C)]
pub struct InstanceData {
  pub model_matrix: [[f32; 4]; 4],
  pub color: [f32; 3],
}

impl<V, I> Model<V, I> {
  pub fn get(&self, handle: usize) -> Option<&I> {
    self.instances.get(*self.handle_to_index.get(&handle)?)
  }

  pub fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
    self.instances.get_mut(*self.handle_to_index.get(&handle)?)
  }

  pub fn swap_by_handle(&mut self, i: usize, j: usize) -> Result<(), InvalidHandle> {
    if i == j {
      return Ok(());
    }
    let i = *self.handle_to_index.get(&i).ok_or(InvalidHandle)?;
    let j = *self.handle_to_index.get(&j).ok_or(InvalidHandle)?;
    self.swap_by_index(i, j);
    Ok(())
  }

  pub fn swap_by_index(&mut self, i: usize, j: usize) {
    if i == j {
      return;
    }
    self.instances.swap(i, j);
    self.handles.swap(i, j);
    self.handle_to_index.insert(self.handles[i], i);
    self.handle_to_index.insert(self.handles[j], j);
  }

  pub fn is_visible(&self, handle: usize) -> Result<bool, InvalidHandle> {
    Ok(*self.handle_to_index.get(&handle).ok_or(InvalidHandle)? < self.first_invisible)
  }

  pub fn make_visible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
    let index = *self.handle_to_index.get(&handle).ok_or(InvalidHandle)?;
    if index >= self.first_invisible {
      self.swap_by_index(index, self.first_invisible);
      self.first_invisible += 1;
    }
    Ok(())
  }

  pub fn make_invisible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
    let index = *self.handle_to_index.get(&handle).ok_or(InvalidHandle)?;
    if index < self.first_invisible {
      self.swap_by_index(index, self.first_invisible - 1);
      self.first_invisible -= 1;
    }
    Ok(())
  }

  pub fn insert(&mut self, instance: I) -> usize {
    let handle = self.next_handle;
    self.next_handle += 1;
    self.handles.push(handle);
    self.instances.push(instance);
    self
      .handle_to_index
      .insert(handle, self.instances.len() - 1);
    handle
  }

  pub fn insert_visibly(&mut self, instance: I) -> usize {
    let handle = self.insert(instance);
    self.make_visible(handle).ok();
    handle
  }

  pub fn remove(&mut self, handle: usize) -> Result<I, InvalidHandle> {
    let index = *self.handle_to_index.get(&handle).ok_or(InvalidHandle)?;
    let instance = self.instances.remove(index);
    self.handles.remove(index);
    self.handle_to_index.remove(&handle);
    if index < self.first_invisible {
      self.first_invisible -= 1;
    }
    Ok(instance)
  }

  pub fn update_vertex_buffer(
    &mut self,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
  ) -> Result<(), vk::Result> {
    let data = self.vertex_data.as_slice();
    if let Some(buffer) = &mut self.vertex_buffer {
      buffer.fill(data)?;
    } else {
      let mut buffer = Buffer::new(
        allocator,
        device,
        std::mem::size_of_val(data) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
      )?;
      buffer.fill(data)?;
      self.vertex_buffer = Some(buffer);
    }
    Ok(())
  }

  pub fn update_instance_buffer(
    &mut self,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
  ) -> Result<(), vk::Result> {
    let data = &self.instances[..self.first_invisible];
    if let Some(buffer) = &mut self.instance_buffer {
      buffer.fill(data)?;
    } else {
      let mut buffer = Buffer::new(
        allocator,
        device,
        std::mem::size_of_val(data) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
      )?;
      buffer.fill(data)?;
      self.instance_buffer = Some(buffer);
    }
    Ok(())
  }

  pub fn draw(&self, logical_device: &ash::Device, command_buffer: vk::CommandBuffer) {
    if let Some(vertex_buffer) = &self.vertex_buffer {
      if let Some(instance_buffer) = &self.instance_buffer {
        if self.first_invisible > 0 {
          unsafe {
            logical_device.cmd_bind_vertex_buffers(
              command_buffer,
              0,
              &[vertex_buffer.buffer],
              &[0],
            );
            logical_device.cmd_bind_vertex_buffers(
              command_buffer,
              1,
              &[instance_buffer.buffer],
              &[0],
            );
            logical_device.cmd_draw(
              command_buffer,
              self.vertex_data.len() as u32,
              self.first_invisible as u32,
              0,
              0,
            );
          }
        }
      }
    }
  }

  pub fn cleanup(&mut self, logical_device: &ash::Device, allocator: &mut vulkan::Allocator) {
    if let Some(buffer) = self.vertex_buffer.take() {
      unsafe {
        logical_device.destroy_buffer(buffer.buffer, None);
        allocator.free(buffer.allocation).unwrap();
      }
    }
    if let Some(buffer) = self.instance_buffer.take() {
      unsafe {
        logical_device.destroy_buffer(buffer.buffer, None);
        allocator.free(buffer.allocation).unwrap();
      }
    }
  }
}

impl Model<[f32; 3], InstanceData> {
  pub fn cube() -> Model<[f32; 3], InstanceData> {
    let lbf = [-1.0, 1.0, 0.0];
    let lbb = [-1.0, 1.0, 1.0];
    let ltf = [-1.0, -1.0, 0.0];
    let ltb = [-1.0, -1.0, 1.0];
    let rbf = [1.0, 1.0, 0.0];
    let rbb = [1.0, 1.0, 1.0];
    let rtf = [1.0, -1.0, 0.0];
    let rtb = [1.0, -1.0, 1.0];

    Model {
      vertex_data: vec![
        lbf, lbb, rbb, lbf, rbb, rbf, //bottom
        ltf, rtb, ltb, ltf, rtf, rtb, //top
        lbf, rtf, ltf, lbf, rbf, rtf, //front
        lbb, ltb, rtb, lbb, rtb, rbb, //back
        lbf, ltf, lbb, lbb, ltf, ltb, //left
        rbf, rbb, rtf, rbb, rtb, rtf, //right
      ],
      handle_to_index: std::collections::HashMap::new(),
      handles: vec![],
      instances: vec![],
      first_invisible: 0,
      next_handle: 0,
      vertex_buffer: None,
      instance_buffer: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct InvalidHandle;

impl std::fmt::Display for InvalidHandle {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Invalid handle")
  }
}

impl std::error::Error for InvalidHandle {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}