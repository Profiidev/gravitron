use game_object::GameObject;

mod game_object;

pub struct Scene {
  name: String,
  game_objects: Vec<Box<dyn GameObject>>,
}

impl Scene {
  pub fn new(name: String) -> Self {
    Self {
      name,
      game_objects: Vec::new(),
    }
  }

  pub fn add_game_object(mut self, game_object: Box<dyn GameObject>) -> Self {
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