use log::trace;
use std::time::Instant;

use crate::ecs::{query::Query, systems::ResMut};

use crate::components::renderer::MeshRenderer;
use crate::vulkan::Vulkan;

pub fn renderer(mut vulkan: ResMut<Vulkan>, to_render: Query<&mut MeshRenderer>) {
  trace!("Executing MeshRenderer");

  let vulkan = &mut *vulkan;
  vulkan.wait_for_draw_start();
  vulkan.testing();

  for e in to_render {
    println!("{:?}", e.x.elapsed());
    e.x = Instant::now();
  }

  vulkan.draw_frame();

  // ! TEMPORARY
  dbg!("Success");
  std::thread::sleep(std::time::Duration::from_secs(2));
  vulkan.destroy();
  std::process::exit(0);
}
