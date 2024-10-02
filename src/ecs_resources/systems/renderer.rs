use log::trace;
use std::time::Instant;

use crate::ecs::{systems::query::Query, systems::resources::ResMut};

use crate::ecs_resources::components::renderer::MeshRenderer;
use crate::vulkan::Vulkan;

pub fn renderer(mut vulkan: ResMut<Vulkan>, to_render: Query<&mut MeshRenderer>) {
  trace!("Executing MeshRenderer");

  let vulkan = &mut *vulkan;
  vulkan.wait_for_draw_start();
  vulkan.record_command_buffer();

  for e in to_render {
    println!("{:?}", e.x.elapsed());
    e.x = Instant::now();
  }

  vulkan.draw_frame();
}
