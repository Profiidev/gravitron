use std::any::Any;

pub struct GameObject {
  id: u32,
  components: Vec<Box<dyn GameObjectComponent>>,
}

impl GameObject {
  pub fn new() -> Self {
    Self {
      id: rand::random(),
      components: Vec::new(),
    }
  }

  pub fn id(&self) -> u32 {
    self.id
  }

  pub fn add_component<T: GameObjectComponent + 'static>(mut self, component: T) -> Self {
    self.components.push(Box::new(component));
    self
  }

  pub fn get_component_mut<T: GameObjectComponent + 'static>(&mut self) -> Option<&mut T> {
    for component in &mut self.components.iter_mut() {
      if let Some(component) = (component as &mut dyn Any).downcast_mut::<T>() {
        return Some(component);
      }
    }

    None
  }

  pub fn get_component<T: GameObjectComponent + 'static>(&self) -> Option<&T> {
    for component in &self.components {
      if let Some(component) = (component as &dyn Any).downcast_ref::<T>() {
        return Some(component);
      }
    }

    None
  }

  pub fn init(&mut self) {
    for component in self.components.iter_mut() {
      component.init();
    }
  }

  pub fn update(&mut self) {
    for component in self.components.iter_mut() {
      component.update();
    }
  }

  pub fn fixed_update(&mut self) {
    for component in self.components.iter_mut() {
      component.fixed_update();
    }
  }
}

impl Default for GameObject {
  fn default() -> Self {
    Self::new()
  }
}

pub trait GameObjectComponent {
  fn init(&mut self) {}
  fn update(&mut self) {}
  fn fixed_update(&mut self) {}
}
