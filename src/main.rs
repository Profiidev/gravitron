use std::time::Instant;

use gravitron::{
  config::EngineConfig, ecs_resources::components::renderer::MeshRenderer, engine::Gravitron,
};

fn main() {
  let config = EngineConfig::default();
  let mut builder = Gravitron::builder(config);
  builder.create_entity(MeshRenderer { x: Instant::now() });
  let engine = builder.build();
  engine.run();
}
