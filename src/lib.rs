pub mod components;
pub mod ecs;
pub mod engine;
pub mod resources;

pub use ecs::Id;

pub use glam as math;
pub use log;

pub use gravitron_utils as utils;

pub mod plugin {
  pub use gravitron_components::ComponentPlugin;
  pub use gravitron_plugin::{app, config::*, Plugin};
  pub use gravitron_renderer::{config::*, RendererPlugin};
  pub use gravitron_window::{config::*, WindowPlugin};
}

pub mod window {
  pub use gravitron_window::winit;
}
