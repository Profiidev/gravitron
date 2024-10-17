use std::collections::HashMap;

use crate::{
  vulkan::memory::{
    manager::MemoryManager,
    types::{BufferBlockSize, BufferId},
    BufferMemory,
  },
  Id,
};
use anyhow::Error;
use ash::vk;
use cube::cube;

mod cube;

pub type ModelId = Id;

pub struct ModelManager {
  models: HashMap<ModelId, Model>,
  last_id: ModelId,
  vertex_buffer: BufferId,
  index_buffer: BufferId,
  instance_buffer: BufferId,
}

pub const CUBE_MODEL: Id = 0;

struct Model {
  vertices: BufferMemory,
  indices: BufferMemory,
  index_len: u32,
  instance_alloc_size: usize,
  instances: HashMap<String, (BufferMemory, Vec<InstanceData>)>,
}

#[derive(Debug)]
#[repr(C)]
pub struct VertexData {
  position: glam::Vec3,
  normal: glam::Vec3,
  uv: glam::Vec2,
}

#[derive(Debug, PartialEq, Clone)]
#[repr(C, packed)]
pub struct InstanceData {
  model_matrix: glam::Mat4,
  inv_model_matrix: glam::Mat4,
  color: glam::Vec3,
  metallic: f32,
  roughness: f32,
  texture_id: u32,
}

pub enum InstanceCount {
  High,
  Medium,
  Low,
  Exact(usize),
}

impl ModelManager {
  pub fn new(memory_manager: &mut MemoryManager) -> Result<Self, Error> {
    let vertex_buffer = memory_manager
      .create_advanced_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, BufferBlockSize::Large)?;
    let index_buffer = memory_manager
      .create_advanced_buffer(vk::BufferUsageFlags::INDEX_BUFFER, BufferBlockSize::Large)?;
    let instance_buffer = memory_manager
      .create_advanced_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, BufferBlockSize::Large)?;

    let mut manager = ModelManager {
      models: HashMap::new(),
      last_id: 0,
      vertex_buffer,
      index_buffer,
      instance_buffer,
    };

    let (vertex_data, index_data) = cube();
    manager
      .add_model(
        memory_manager,
        vertex_data,
        index_data,
        InstanceCount::Medium,
      )
      .unwrap();

    Ok(manager)
  }

  pub fn add_model(
    &mut self,
    memory_manager: &mut MemoryManager,
    vertex_data: Vec<VertexData>,
    index_data: Vec<u32>,
    instance_count: InstanceCount,
  ) -> Option<(ModelId, bool)> {
    let vertices_slice = vertex_data.as_slice();
    let (vertices, vert_resized) =
      memory_manager.add_to_buffer(self.vertex_buffer, vertices_slice)?;
    let index_slice = index_data.as_slice();
    let (indices, index_resized) = memory_manager.add_to_buffer(self.index_buffer, index_slice)?;

    let id = self.last_id;
    self.models.insert(
      id,
      Model::new(vertices, indices, index_data.len() as u32, instance_count),
    );
    self.last_id += 1;

    Some((id, vert_resized || index_resized))
  }

  pub fn record_command_buffer(
    &self,
    memory_manager: &MemoryManager,
    command_buffer: vk::CommandBuffer,
    device: &ash::Device,
  ) {
    let vertex_buffer = memory_manager.get_vk_buffer(self.vertex_buffer).unwrap();
    let index_buffer = memory_manager.get_vk_buffer(self.index_buffer).unwrap();
    let instance_buffer = memory_manager.get_vk_buffer(self.instance_buffer).unwrap();
    unsafe {
      device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
      device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
      device.cmd_bind_vertex_buffers(command_buffer, 1, &[instance_buffer], &[0]);
    }
  }

  pub fn update_draw_buffer(
    &mut self,
    cmd_buffer: BufferId,
    commands: &mut HashMap<ModelId, HashMap<String, (vk::DrawIndexedIndirectCommand, u64)>>,
    memory_manager: &mut MemoryManager,
    instances: HashMap<ModelId, HashMap<String, Vec<InstanceData>>>,
  ) -> (
    HashMap<String, Vec<(ModelId, vk::DrawIndexedIndirectCommand)>>,
    bool,
  ) {
    let instance_size = std::mem::size_of::<InstanceData>();
    let mut copy_offset = 0;
    let mut instance_copies_info = Vec::new();
    let mut instance_copies = Vec::new();

    let cmd_size = size_of::<vk::DrawIndexedIndirectCommand>() as u64;
    let mut cmd_copies_info = Vec::new();
    let mut cmd_copies = Vec::new();
    let mut cmd_new = HashMap::new();

    let mut buffer_resized = false;

    for (model_id, shaders) in commands.iter_mut() {
      if !instances.contains_key(model_id) {
        for (cmd, offset) in shaders.values_mut() {
          if cmd.instance_count > 0 {
            cmd.instance_count = 0;
            cmd_copies_info.push(vk::BufferCopy {
              size: cmd_size,
              src_offset: cmd_copies.len() as u64 * cmd_size,
              dst_offset: *offset,
            });
            cmd_copies.push(*cmd);
          }
        }
      }
    }

    for (model_id, shaders) in instances {
      let Some(model) = self.models.get_mut(&model_id) else {
        continue;
      };
      let model_commands = commands.entry(model_id).or_default();

      for (shader, (cmd, offset)) in model_commands.iter_mut() {
        if !shaders.contains_key(shader) && cmd.instance_count > 0 {
          cmd.instance_count = 0;
          cmd_copies_info.push(vk::BufferCopy {
            size: cmd_size,
            src_offset: cmd_copies.len() as u64 * cmd_size,
            dst_offset: *offset,
          });
          cmd_copies.push(*cmd);
        }
      }

      for (shader, instances) in shaders {
        if let Some((mem, model_instances)) = model.instances.get_mut(&shader) {
          let (command, offset) = model_commands.get_mut(&shader).unwrap();

          if model_instances.len() != instances.len() {
            command.instance_count = instances.len() as u32;
            cmd_copies_info.push(vk::BufferCopy {
              size: cmd_size,
              src_offset: cmd_copies.len() as u64 * cmd_size,
              dst_offset: *offset,
            });
            cmd_copies.push(*command);

            let instances_size = instance_size * instances.len();
            if instances_size > mem.size() {
              let new_size = (instances_size as f32 / model.instance_alloc_size as f32).ceil()
                as usize
                * model.instance_alloc_size;

              buffer_resized =
                buffer_resized || memory_manager.resize_buffer_mem(mem, new_size).unwrap();
              command.first_instance = (mem.offset() / instance_size) as u32;
            }
          }

          let mut to_copy = Vec::new();
          for (i, instance) in instances.iter().enumerate() {
            if let Some(other_instance) = model_instances.get_mut(i) {
              if other_instance == instance && !to_copy.is_empty() {
                let copy_size = (instance_size * to_copy.len()) as u64;

                instance_copies_info.push(vk::BufferCopy {
                  dst_offset: (mem.offset() + instance_size * (i - to_copy.len())) as u64,
                  src_offset: copy_offset,
                  size: copy_size,
                });
                instance_copies.extend(to_copy);

                copy_offset += copy_size;
                to_copy = Vec::new();
              } else if other_instance != instance {
                to_copy.push(instance.clone());
                *other_instance = instance.clone();
              }
            } else {
              to_copy.push(instance.clone());
              model_instances.push(instance.clone());
            };
          }

          if !to_copy.is_empty() {
            let copy_size = (std::mem::size_of::<InstanceData>() * to_copy.len()) as u64;

            instance_copies_info.push(vk::BufferCopy {
              dst_offset: (mem.offset()
                + std::mem::size_of::<InstanceData>() * (instances.len() - to_copy.len()))
                as u64,
              src_offset: copy_offset,
              size: copy_size,
            });
            instance_copies.extend(to_copy);

            copy_offset += copy_size;
          }
        } else {
          let instances_size = instance_size * instances.len();
          let required_size = (instances_size as f32 / model.instance_alloc_size as f32).ceil()
            as usize
            * model.instance_alloc_size;
          let Some((mem, buffer_resize_local)) =
            memory_manager.reserve_buffer_mem(self.instance_buffer, required_size)
          else {
            continue;
          };

          buffer_resized = buffer_resized || buffer_resize_local;

          let instances_slice = instances.as_slice();
          memory_manager.write_to_buffer(&mem, instances_slice);

          let cmd = vk::DrawIndexedIndirectCommand {
            index_count: model.index_len,
            instance_count: instances.len() as u32,
            first_index: model.indices.offset() as u32,
            vertex_offset: model.vertices.offset() as i32,
            first_instance: (mem.offset() / instance_size) as u32,
          };

          let shader_cmd: &mut Vec<(u64, vk::DrawIndexedIndirectCommand)> =
            cmd_new.entry(shader.clone()).or_default();
          shader_cmd.push((model_id, cmd));

          model.instances.insert(shader, (mem, instances));
        }
      }
    }

    if !instance_copies.is_empty() {
      memory_manager.write_to_buffer_direct(
        self.instance_buffer,
        &instance_copies,
        &instance_copies_info,
      );
    }

    if !cmd_copies.is_empty() {
      memory_manager.write_to_buffer_direct(cmd_buffer, &cmd_copies, &cmd_copies_info);
    }

    (cmd_new, buffer_resized)
  }
}

impl Model {
  fn new(
    vertices: BufferMemory,
    indices: BufferMemory,
    index_len: u32,
    instance_count: InstanceCount,
  ) -> Self {
    Self {
      vertices,
      index_len,
      indices,
      instance_alloc_size: usize::from(instance_count) * std::mem::size_of::<InstanceData>(),
      instances: HashMap::new(),
    }
  }
}

impl InstanceData {
  pub fn new(
    model_matrix: glam::Mat4,
    inv_model_matrix: glam::Mat4,
    color: glam::Vec3,
    metallic: f32,
    roughness: f32,
    texture_id: u32,
  ) -> Self {
    Self {
      model_matrix,
      inv_model_matrix,
      color,
      metallic,
      roughness,
      texture_id,
    }
  }
}

impl From<InstanceCount> for usize {
  fn from(value: InstanceCount) -> Self {
    match value {
      InstanceCount::High => 1000,
      InstanceCount::Medium => 100,
      InstanceCount::Low => 10,
      InstanceCount::Exact(count) => count,
    }
  }
}
