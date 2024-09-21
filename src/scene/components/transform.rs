use glam as g;

use crate::scene::game_object::GameObjectComponent;

pub struct Transform {
  position: g::Vec3,
  rotation: g::Quat,
  scale: g::Vec3,
}

impl Transform {
  pub fn new(position: g::Vec3, rotation: g::Quat, scale: g::Vec3) -> Self {
    Self {
      position,
      rotation,
      scale,
    }
  }
}

impl GameObjectComponent for Transform {}
