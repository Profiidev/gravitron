use game_object::GameObject;

use crate::util::mutator::Mutator;

pub mod components;
pub mod game_object;

pub struct Scene {
  id: u32,
  game_objects: Vec<Mutator<GameObject>>,
}

impl Scene {
  pub fn new() -> Self {
    Self {
      id: rand::random(),
      game_objects: Vec::new(),
    }
  }

  pub fn add_game_object(mut self, game_object: GameObject) -> Self {
    self.game_objects.push(Mutator::new(game_object));
    self
  }

  pub fn init(&mut self) {
    for game_object in self.game_objects.iter() {
      game_object.get_mut().init();
    }
  }

  pub fn update(&mut self) {
    for game_object in self.game_objects.iter() {
      game_object.get_mut().update();
    }
  }

  pub fn game_objects(&self) -> &Vec<Mutator<GameObject>> {
    &self.game_objects
  }
}

impl Default for Scene {
  fn default() -> Self {
    Self::new()
  }
}
