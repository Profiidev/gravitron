use ecs::{
  resources::{cleanup_resource, Resources},
  systems::renderer::{execute_renderer, init_renderer, renderer_recording},
};
use gravitron_components::ComponentPlugin;
use gravitron_plugin::{
  app::{App, AppBuilder, Build, Cleanup, Finalize},
  stages::MainSystemStage,
  Plugin,
};
#[cfg(target_os = "linux")]
use gravitron_window::ecs::resources::event_loop::EventLoop;
use gravitron_window::WindowPlugin;
use log::debug;
pub use vk_shader_macros::{glsl, include_glsl};

#[cfg(feature = "debug")]
mod debug;
mod device;
pub mod ecs;
mod error;
mod graphics;
mod instance;
mod memory;
mod pipeline;
mod surface;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_main_system_at_stage(init_renderer, MainSystemStage::RenderInit);
    builder.add_main_system_at_stage(renderer_recording, MainSystemStage::RenderRecording);
    builder.add_main_system_at_stage(execute_renderer, MainSystemStage::RenderExecute);
  }

  fn finalize(&self, builder: &mut AppBuilder<Finalize>) {
    let config = builder.config();
    let window = builder
      .get_resource()
      .expect("Error: Window Plugin must be initialized before the Renderer Plugin");

    #[cfg(target_os = "linux")]
    let event_loop = builder
      .get_resource::<EventLoop>()
      .expect("Error: Window Plugin must be initialized before the Renderer Plugin");

    Resources::create(
      config.vulkan.clone(),
      config,
      window,
      #[cfg(target_os = "linux")]
      event_loop.wayland(),
    )
    .expect("Error: Failed to create Renderer resources")
    .add_resources(builder);
  }

  fn cleanup(&self, app: &mut App<Cleanup>) {
    debug!("Cleaning up Renderer Resources");
    cleanup_resource(app).expect("Failed to cleanup Renderer resources");
  }

  fn dependencies(&self) -> Vec<gravitron_plugin::PluginID> {
    vec![WindowPlugin.id(), ComponentPlugin.id()]
  }
}
