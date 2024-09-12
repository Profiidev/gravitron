use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct Signal<T = bool> {
  value: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl Signal {
  pub fn signal(&self) {
    self.send(true);
  }
}

impl<T> Signal<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn send(&self, data: T) {
    let (lock, cvar) = &*self.value;
    let mut started = lock.lock().unwrap();
    *started = Some(data);
    cvar.notify_all();
  }

  pub fn wait(&self) -> T {
    let (lock, cvar) = &*self.value;
    let started = lock.lock().unwrap();
    let mut data = cvar
      .wait_while(started, |started| started.is_none())
      .unwrap();
    data.take().unwrap()
  }

  pub fn is_signaled(&self) -> bool {
    let (lock, _) = &*self.value;
    let started = lock.lock().unwrap();
    started.is_some()
  }
}

impl<T> Default for Signal<T> {
  fn default() -> Self {
    Self {
      value: Arc::new((Mutex::new(None), Condvar::new())),
    }
  }
}
