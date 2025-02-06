use std::collections::HashMap;

use crate::{memory::types::BufferMemory, pipeline::manager::GraphicsPipelineHandle};

pub const CUBE_MODEL: ModelHandle = ModelHandle(0);
pub const PLANE_MODEL: ModelHandle = ModelHandle(1);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Default)]
pub struct ModelHandle(pub(crate) u64);

pub(crate) struct Model {
  pub(crate) vertices: BufferMemory,
  pub(crate) indices: BufferMemory,
  pub(crate) index_len: u32,
  pub(crate) instance_alloc_size: usize,
  pub(crate) instances: HashMap<GraphicsPipelineHandle, (BufferMemory, Vec<InstanceData>)>,
}

#[derive(Debug)]
#[repr(C)]
pub struct VertexData {
  pub position: glam::Vec3,
  pub normal: glam::Vec3,
  pub uv: glam::Vec2,
}

#[derive(Debug, PartialEq, Clone)]
#[repr(C, packed)]
pub struct InstanceData {
  pub model_matrix: glam::Mat4,
  pub inv_model_matrix: glam::Mat4,
  pub color: glam::Vec4,
  pub metallic: f32,
  pub roughness: f32,
  pub texture_id: u32,
}

pub enum InstanceCount {
  High,
  Medium,
  Low,
  Exact(usize),
}

impl Model {
  pub fn new(
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
    color: glam::Vec4,
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
