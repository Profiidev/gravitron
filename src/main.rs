use std::time::Instant;

use gravitron::{components::renderer::MeshRenderer, config::EngineConfig, engine::Gravitron};

fn main() {
  let config = EngineConfig::default();
  let mut builder = Gravitron::builder(config);
  builder.create_entity(MeshRenderer { x: Instant::now() });
  let engine = builder.build();
  engine.run();
}
