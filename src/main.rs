use std::time::Instant;

use gravitron::{
  components::renderer::MeshRenderer,
  config::{
    vulkan::{RendererConfig, VulkanConfig},
    EngineConfig,
  },
  engine::Gravitron,
};

fn main() {
  let config = EngineConfig::default().set_vulkan_config(
    VulkanConfig::default().set_renderer_config(RendererConfig::default().set_debug(true)),
  );
  let mut builder = Gravitron::builder(config);
  builder.create_entity(MeshRenderer { x: Instant::now() });
  let engine = builder.build();
  engine.run();
}
