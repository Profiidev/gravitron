use config::WindowConfig;
use ecs::{
  resources::{event_loop::EventLoop, handle::WindowHandle, input::Input},
  systems::input_update::update_input,
};
use gravitron_plugin::{
  app::{AppBuilder, Build, Finalize},
  stages::MainSystemStage,
  Plugin,
};

pub use winit;

pub mod config;
pub mod ecs;
mod window;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_config(WindowConfig::default());
    builder.add_main_system_at_stage(update_input, MainSystemStage::PreRender);
  }

  fn finalize(&self, builder: &mut AppBuilder<Finalize>) {
    let config = builder.config::<WindowConfig>().unwrap();
    let (event_loop, window) = EventLoop::init(config.clone());

    builder.add_resource(event_loop);
    builder.add_resource(WindowHandle::new(&window).expect("Failed to create window handle"));
    builder.add_resource(window);
    builder.add_resource(Input::default());
  }
}
