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

  pub fn mesh_id(&self) -> u32 {
    self.mesh_id
  }

  pub fn material_id(&self) -> u32 {
    self.material_id
  }
}

impl GameObjectComponent for MeshRenderer {
  fn init(&mut self) {
    println!("MeshRenderer init");
  }
}
