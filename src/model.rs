use ash::vk;
use gpu_allocator::vulkan;
use glam as g;

use crate::buffer::Buffer;

pub struct Model<V, I> {
  pub vertex_data: Vec<V>,
  pub index_data: Vec<u32>,
  pub handle_to_index: std::collections::HashMap<usize, usize>,
  pub handles: Vec<usize>,
  pub instances: Vec<I>,
  pub first_invisible: usize,
  pub next_handle: usize,
  pub vertex_buffer: Option<Buffer>,
  pub index_buffer: Option<Buffer>,
  pub instance_buffer: Option<Buffer>,
}

#[repr(C)]
pub struct InstanceData {
  pub model_matrix: [[f32; 4]; 4],
  pub inv_model_matrix: [[f32; 4]; 4],
  pub color: [f32; 3],
  pub metallic: f32,
  pub roughness: f32,
}

impl InstanceData {
  pub fn new(model_matrix: g::Mat4, color: [f32; 3], metallic: f32, roughness: f32) -> Self {
    Self {
      model_matrix: model_matrix.to_cols_array_2d(),
      inv_model_matrix: model_matrix.inverse().to_cols_array_2d(),
      color,
      metallic,
      roughness,
    }
  }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VertexData {
  pub position: [f32; 3],
  pub normal: [f32; 3],
}

impl VertexData {
  fn midpoint(a: &VertexData, b: &VertexData) -> VertexData {
    VertexData {
      position: [
        0.5 * (a.position[0] + b.position[0]),
        0.5 * (a.position[1] + b.position[1]),
        0.5 * (a.position[2] + b.position[2]),
      ],
      normal: normalize([
        0.5 * (a.normal[0] + b.normal[0]),
        0.5 * (a.normal[1] + b.normal[1]),
        0.5 * (a.normal[2] + b.normal[2]),
      ]),
    }
  }
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
  let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
  [v[0] / l, v[1] / l, v[2] / l]
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

  pub fn update_index_buffer(
    &mut self,
    allocator: &mut vulkan::Allocator,
    device: &ash::Device,
  ) -> Result<(), vk::Result> {
    let data = self.index_data.as_slice();
    if let Some(buffer) = &mut self.index_buffer {
      buffer.fill(data)?;
    } else {
      let mut buffer = Buffer::new(
        allocator,
        device,
        std::mem::size_of_val(data) as u64,
        vk::BufferUsageFlags::INDEX_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
      )?;
      buffer.fill(data)?;
      self.index_buffer = Some(buffer);
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
      if let Some(index_buffer) = &self.index_buffer {
        if let Some(instance_buffer) = &self.instance_buffer {
          if self.first_invisible > 0 {
            unsafe {
              logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &[vertex_buffer.buffer],
                &[0],
              );
              logical_device.cmd_bind_index_buffer(
                command_buffer,
                index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
              );
              logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                1,
                &[instance_buffer.buffer],
                &[0],
              );
              logical_device.cmd_draw_indexed(
                command_buffer,
                self.index_data.len() as u32,
                self.first_invisible as u32,
                0,
                0,
                0,
              );
            }
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
    if let Some(buffer) = self.index_buffer.take() {
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

impl Model<VertexData, InstanceData> {
  pub fn cube() -> Model<VertexData, InstanceData> {
    let lbf = VertexData {
      position: [-1.0, 1.0, -1.0],
      normal: [0.0, 0.0, -1.0],
    };
    let lbb = VertexData {
      position: [-1.0, 1.0, 1.0],
      normal: [0.0, 0.0, 1.0],
    };
    let ltf = VertexData {
      position: [-1.0, -1.0, -1.0],
      normal: [0.0, 0.0, -1.0],
    };
    let ltb = VertexData {
      position: [-1.0, -1.0, 1.0],
      normal: [0.0, 0.0, 1.0],
    };
    let rbf = VertexData {
      position: [1.0, 1.0, -1.0],
      normal: [0.0, 0.0, -1.0],
    };
    let rbb = VertexData {
      position: [1.0, 1.0, 1.0],
      normal: [0.0, 0.0, 1.0],
    };
    let rtf = VertexData {
      position: [1.0, -1.0, -1.0],
      normal: [0.0, 0.0, -1.0],
    };
    let rtb = VertexData {
      position: [1.0, -1.0, 1.0],
      normal: [0.0, 0.0, 1.0],
    };

    Model {
      vertex_data: vec![lbf, lbb, ltf, ltb, rbf, rbb, rtf, rtb],
      index_data: vec![
        0, 1, 5, 0, 5, 4, //bottom
        2, 7, 3, 2, 6, 7, //top
        0, 6, 2, 0, 4, 6, //front
        1, 3, 7, 1, 7, 5, //back
        0, 2, 1, 1, 2, 3, //left
        4, 5, 6, 5, 7, 6, //right
      ],
      handle_to_index: std::collections::HashMap::new(),
      handles: vec![],
      instances: vec![],
      first_invisible: 0,
      next_handle: 0,
      vertex_buffer: None,
      index_buffer: None,
      instance_buffer: None,
    }
  }

  pub fn sphere(refinements: u32) -> Model<VertexData, InstanceData> {
    let mut model = Self::icosphere();
    for _ in 0..refinements {
      model.refine();
    }
    for v in &mut model.vertex_data {
      v.position = normalize(v.position);
    }
    model
  }

  fn icosphere() -> Model<VertexData, InstanceData> {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let ft_1 = VertexData {
      position: [-phi, 1.0, 0.0],
      normal: normalize([-phi, 1.0, 0.0]),
    };
    let fb_1 = VertexData {
      position: [-phi, -1.0, 0.0],
      normal: normalize([-phi, -1.0, 0.0]),
    };
    let bt_1 = VertexData {
      position: [phi, 1.0, 0.0],
      normal: normalize([phi, 1.0, 0.0]),
    };
    let bb_1 = VertexData {
      position: [phi, -1.0, 0.0],
      normal: normalize([phi, -1.0, 0.0]),
    };
    let fr_2 = VertexData {
      position: [-1.0, 0.0, phi],
      normal: normalize([-1.0, 0.0, phi]),
    };
    let fl_2 = VertexData {
      position: [1.0, 0.0, phi],
      normal: normalize([1.0, 0.0, phi]),
    };
    let br_2 = VertexData {
      position: [-1.0, 0.0, -phi],
      normal: normalize([-1.0, 0.0, -phi]),
    };
    let bl_2 = VertexData {
      position: [1.0, 0.0, -phi],
      normal: normalize([1.0, 0.0, -phi]),
    };
    let tl_3 = VertexData {
      position: [0.0, phi, 1.0],
      normal: normalize([0.0, phi, 1.0]),
    };
    let tr_3 = VertexData {
      position: [0.0, phi, -1.0],
      normal: normalize([0.0, phi, -1.0]),
    };
    let bl_3 = VertexData {
      position: [0.0, -phi, 1.0],
      normal: normalize([0.0, -phi, 1.0]),
    };
    let br_3 = VertexData {
      position: [0.0, -phi, -1.0],
      normal: normalize([0.0, -phi, -1.0]),
    };

    Self {
      vertex_data: vec![
        ft_1, fb_1, bt_1, bb_1, fr_2, fl_2, br_2, bl_2, tl_3, tr_3, bl_3, br_3,
      ],
      index_data: vec![
        0, 9, 8, //
        0, 8, 4, //
        0, 4, 1, //
        0, 1, 6, //
        0, 6, 9, //
        8, 9, 2, //
        8, 2, 5, //
        8, 5, 4, //
        4, 5, 10, //
        4, 10, 1, //
        1, 10, 11, //
        1, 11, 6, //
        2, 3, 5, //
        2, 7, 3, //
        2, 9, 7, //
        5, 3, 10, //
        3, 11, 10, //
        3, 7, 11, //
        6, 7, 9, //
        6, 11, 7, //
      ],
      handle_to_index: std::collections::HashMap::new(),
      handles: vec![],
      instances: vec![],
      first_invisible: 0,
      next_handle: 0,
      vertex_buffer: None,
      index_buffer: None,
      instance_buffer: None,
    }
  }

  fn refine(&mut self) {
    let mut new_indices = vec![];
    let mut midpoints = std::collections::HashMap::<(u32, u32), u32>::new();
    for triangle in self.index_data.chunks(3) {
      let a = triangle[0];
      let b = triangle[1];
      let c = triangle[2];
      let vertex_a = self.vertex_data[a as usize];
      let vertex_b = self.vertex_data[b as usize];
      let vertex_c = self.vertex_data[c as usize];
      let mab = if let Some(ab) = midpoints.get(&(a, b)) {
        *ab
      } else {
        let vertex_ab = VertexData::midpoint(&vertex_a, &vertex_b);
        let mab = self.vertex_data.len() as u32;
        self.vertex_data.push(vertex_ab);
        midpoints.insert((a, b), mab);
        midpoints.insert((b, a), mab);
        mab
      };
      let mbc = if let Some(bc) = midpoints.get(&(b, c)) {
        *bc
      } else {
        let vertex_bc = VertexData::midpoint(&vertex_b, &vertex_c);
        let mbc = self.vertex_data.len() as u32;
        midpoints.insert((b, c), mbc);
        midpoints.insert((c, b), mbc);
        self.vertex_data.push(vertex_bc);
        mbc
      };
      let mca = if let Some(ca) = midpoints.get(&(c, a)) {
        *ca
      } else {
        let vertex_ca = VertexData::midpoint(&vertex_c, &vertex_a);
        let mca = self.vertex_data.len() as u32;
        midpoints.insert((c, a), mca);
        midpoints.insert((a, c), mca);
        self.vertex_data.push(vertex_ca);
        mca
      };
      new_indices.extend_from_slice(&[mca, a, mab, mab, b, mbc, mbc, c, mca, mab, mbc, mca]);
    }
    self.index_data = new_indices;
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
