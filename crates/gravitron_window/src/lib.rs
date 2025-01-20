use ecs::resources::{event_loop::EventLoop, handle::WindowHandle};
use gravitron_plugin::{
  app::{AppBuilder, Finalize},
  Plugin,
};

pub mod ecs;
mod window;

pub struct WindowPlugin {}

impl Plugin for WindowPlugin {
  fn finalize(&self, builder: &mut AppBuilder<Finalize>) {
    let (event_loop, window) = EventLoop::init(builder.config().window.clone());

    builder.add_resource(event_loop);
    builder.add_resource(WindowHandle::new(&window));
    builder.add_resource(window);
  }
}
