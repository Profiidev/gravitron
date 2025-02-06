use config::VulkanConfig;
use ecs::{
  resources::{cleanup_resource, Resources},
  systems::{
    descriptor::{reset_descriptors, update_default_descriptors, update_descriptors},
    memory::reset_buffer_reallocated,
    pipeline::pipeline_changed_reset,
    renderer::{draw_data_update, execute_renderer, init_renderer, renderer_recording},
  },
};
use gravitron_components::ComponentPlugin;
use gravitron_plugin::{
  app::{App, AppBuilder, Build, Cleanup, Finalize},
  config::AppConfig,
  stages::MainSystemStage,
  Plugin,
};
#[cfg(target_os = "linux")]
use gravitron_window::ecs::resources::event_loop::EventLoop;
use gravitron_window::{config::WindowConfig, WindowPlugin};
use log::debug;

pub mod config;
#[cfg(feature = "debug")]
mod debug;
mod device;
pub mod ecs;
mod error;
mod instance;
mod memory;
mod model;
mod pipeline;
mod renderer;
mod surface;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_config(VulkanConfig::default());
    builder.add_main_system_at_stage(init_renderer, MainSystemStage::RenderInit);
    builder.add_main_system_at_stage(update_default_descriptors, MainSystemStage::RenderInit);
    builder.add_main_system_at_stage(draw_data_update, MainSystemStage::RenderInit);
    builder.add_main_system_at_stage(update_descriptors, MainSystemStage::RenderPrepare);
    builder.add_main_system_at_stage(renderer_recording, MainSystemStage::RenderRecording);
    builder.add_main_system_at_stage(execute_renderer, MainSystemStage::RenderExecute);
    builder.add_main_system_at_stage(reset_buffer_reallocated, MainSystemStage::PostRender);
    builder.add_main_system_at_stage(pipeline_changed_reset, MainSystemStage::PostRender);
    builder.add_main_system_at_stage(reset_descriptors, MainSystemStage::PostRender);
  }

  fn finalize(&self, builder: &mut AppBuilder<Finalize>) {
    let app_config = builder
      .config::<AppConfig>()
      .expect("Error: Failed to get AppConfig");
    let window_config = builder
      .config::<WindowConfig>()
      .expect("Error: Failed to get AppConfig");
    let config = builder
      .config::<VulkanConfig>()
      .expect("Error: Failed to get Vulkan Config");

    let window = builder
      .get_resource()
      .expect("Error: Window Plugin must be initialized before the Renderer Plugin");

    #[cfg(target_os = "linux")]
    let event_loop = builder
      .get_resource::<EventLoop>()
      .expect("Error: Window Plugin must be initialized before the Renderer Plugin");

    Resources::create(
      config.clone(),
      app_config,
      window_config,
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
