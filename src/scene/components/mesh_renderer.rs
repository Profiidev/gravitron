use crate::scene::game_object::GameObjectComponent;

pub struct MeshRenderer {
  mesh_id: u32,
  material_id: u32,
}

impl MeshRenderer {
  pub fn new(mesh_id: u32, material_id: u32) -> Self {
    Self {
      mesh_id,
      material_id,
    }
  }
}

impl GameObjectComponent for MeshRenderer {
  fn init(&mut self) {
    println!("MeshRenderer init");
  }

  fn update(&mut self) {
    println!("MeshRenderer update");
  }

  fn fixed_update(&mut self) {
    println!("MeshRenderer fixed_update");
  }
}