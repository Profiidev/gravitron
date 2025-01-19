use ecs::systems::renderer::{execute_renderer, init_renderer, renderer_recording};
use gravitron_plugin::{
  app::{AppBuilder, Build},
  stages::SystemStage,
  Plugin,
};
pub use vk_shader_macros::{glsl, include_glsl};

#[cfg(feature = "debug")]
mod debug;
mod device;
pub mod ecs;
pub mod error;
pub mod graphics;
mod instance;
pub mod memory;
mod pipeline;
mod surface;

pub struct RendererPlugin {}

impl Plugin for RendererPlugin {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_main_system_at_stage(init_renderer, SystemStage::RenderInit);
    builder.add_main_system_at_stage(renderer_recording, SystemStage::RenderRecording);
    builder.add_main_system_at_stage(execute_renderer, SystemStage::RenderExecute);
  }
}
