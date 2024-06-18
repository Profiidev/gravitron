use std::{sync::{Arc, Condvar, Mutex}, time::Duration};

#[derive(Clone)]
pub struct Signal {
  value: Arc<(Mutex<bool>, Condvar)>,
}

impl Signal {
  pub fn new() -> Self {
    Signal {
      value: Arc::new((Mutex::new(false), Condvar::new())),
    }
  }

  pub fn signal(&self) {
    let (lock, cvar) = &*self.value;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_all();
  }

  pub fn wait(&self) {
    let (lock, cvar) = &*self.value;
    let started = lock.lock().unwrap();
    let _ = cvar.wait_timeout_while(started, Duration::from_micros(0), |&mut started| !started).unwrap();
  }
}