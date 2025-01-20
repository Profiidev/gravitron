use gravitron_ecs::systems::resources::ResMut;
use gravitron_plugin::ecs::resources::engine_commands::EngineCommands;
use winit::event::WindowEvent;

use crate::ecs::resources::{event_loop::EventLoop, input::Input};

pub fn update_input(
  mut event_loop: ResMut<EventLoop>,
  mut input: ResMut<Input>,
  mut cmds: ResMut<EngineCommands>,
) {
  event_loop.update_events();

  for event in event_loop.events() {
    input.handle_event(event);

    if let WindowEvent::CloseRequested = event {
      cmds.shutdown();
    }
  }
}
