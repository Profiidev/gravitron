use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Mutator<T> {
  data: Arc<Mutex<T>>,
}

impl<T> Mutator<T> {
  pub fn new(data: T) -> Self {
    Mutator {
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn set(&self, data: T) {
    *self.data.lock().unwrap() = data;
  }

  pub fn get(&'_ self) -> std::sync::MutexGuard<'_, T> {
    self.data.lock().unwrap()
  }

  pub fn inner_clone(mutator: &Mutator<T>) -> Mutator<T> {
    Mutator {
      data: Arc::clone(&mutator.data),
    }
  }
}
