use std::time::Instant;

use crate::ecs::Component;

#[derive(Component, Debug)]
pub struct MeshRenderer {
  pub x: Instant,
}
