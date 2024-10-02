use gravitron_ecs::ECSBuilder;
use log::debug;
use renderer::renderer;

mod renderer;

pub fn add_systems(ecs: &mut ECSBuilder) {
  debug!("Adding Engine Systems");

  ecs.add_system(renderer);
}
