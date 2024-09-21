use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Mutator<T> {
  data: Arc<RwLock<T>>,
}

impl<T> Mutator<T> {
  pub fn new(data: T) -> Self {
    Mutator {
      data: Arc::new(RwLock::new(data)),
    }
  }

  pub fn set(&self, data: T) {
    *self.data.write().unwrap() = data;
  }

  pub fn get_mut(&self) -> std::sync::RwLockWriteGuard<T> {
    self.data.write().unwrap()
  }

  pub fn get(&self) -> std::sync::RwLockReadGuard<T> {
    self.data.read().unwrap()
  }
}
