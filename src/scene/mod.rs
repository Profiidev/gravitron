use game_object::GameObject;

mod game_object;
mod components;

pub struct Scene {
  id: u32,
  game_objects: Vec<GameObject>,
}

impl Scene {
  pub fn new() -> Self {
    Self {
      id: rand::random(),
      game_objects: Vec::new(),
    }
  }

  pub fn add_game_object(mut self, game_object: GameObject) -> Self {
    self.game_objects.push(game_object);
    self
  }

  pub fn init(&mut self) {
    for game_object in self.game_objects.iter_mut() {
      game_object.init();
    }
  }

  pub fn update(&mut self) {
    for game_object in self.game_objects.iter_mut() {
      game_object.update();
    }
  }
}

impl Default for Scene {
  fn default() -> Self {
    Self::new()
  }
}
