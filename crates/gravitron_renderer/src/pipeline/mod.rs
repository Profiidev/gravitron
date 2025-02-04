pub mod descriptor;
pub mod graphics;
pub mod manager;
pub(crate) mod pools;

pub use descriptor::manager::DescriptorManager;
pub use manager::PipelineManager;

pub use vk_shader_macros::include_glsl;
