use std::sync::Arc;

use crate::{
  scene::{components::mesh_renderer::MeshRenderer, game_object::GameObject, Scene},
  util::mutator::Mutator,
};

pub struct Renderer {
  tracked_game_objects: Vec<Mutator<GameObject>>,
  scene: Arc<Scene>,
  previous_frame: Vec<(u32, u32)>,
}

impl Renderer {
  pub fn init(scene: Arc<Scene>) -> Self {
    Renderer {
      tracked_game_objects: Vec::new(),
      scene,
      previous_frame: Vec::new(),
    }
  }

  pub fn add_game_object(&mut self, game_object: Mutator<GameObject>) {
    self.tracked_game_objects.push(game_object);
  }

  pub fn remove_game_object(&mut self, id: u32) {
    self.tracked_game_objects.retain(|x| x.get().id() != id);
  }

  pub fn update(&mut self) {
    let mut to_render = Vec::new();
    for game_object in self.tracked_game_objects.iter() {
      let game_object = game_object.get();
      let mesh_renderer = game_object.get_component::<MeshRenderer>();
      if let Some(mesh_renderer) = mesh_renderer {
        to_render.push((mesh_renderer.mesh_id(), mesh_renderer.material_id()));
      }
    }

    if self.previous_frame != to_render {
      self.previous_frame = to_render;
      println!("Render frame");
    }
  }
}
