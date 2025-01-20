use ecs::{
  resources::{event_loop::EventLoop, handle::WindowHandle},
  systems::input_update::update_input,
};
use gravitron_plugin::{
  app::{AppBuilder, Build, Finalize},
  stages::MainSystemStage,
  Plugin,
};

pub mod ecs;
mod window;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_main_system_at_stage(update_input, MainSystemStage::PreRender);
  }

  fn finalize(&self, builder: &mut AppBuilder<Finalize>) {
    let (event_loop, window) = EventLoop::init(builder.config().window.clone());

    builder.add_resource(event_loop);
    builder.add_resource(WindowHandle::new(&window));
    builder.add_resource(window);
  }
}
