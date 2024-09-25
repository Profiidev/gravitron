use std::{sync::mpsc, thread};

use crate::thread::Mutator;

enum Message {
  NewJob(Job),
  Terminate,
}

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
  pub fn new(size: usize) -> Self {
    assert!(size > 0);

    let (sender, receiver) = mpsc::channel();
    let receiver = Mutator::new(receiver);

    let mut workers = Vec::with_capacity(size);

    for _ in 0..size {
      workers.push(Worker::new(Mutator::inner_clone(&receiver)));
    }

    ThreadPool { workers, sender }
  }

  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);

    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    for _ in &mut self.workers {
      self.sender.send(Message::Terminate).unwrap();
    }

    while let Some(worker) = self.workers.pop() {
      worker.thread.join().unwrap()
    }
  }
}

struct Worker {
  thread: thread::JoinHandle<()>,
}

impl Worker {
  fn new(receiver: Mutator<mpsc::Receiver<Message>>) -> Self {
    let thread = thread::spawn(move || loop {
      let message = receiver.get().recv().unwrap();

      match message {
        Message::NewJob(job) => {
          job();
        }
        Message::Terminate => {
          break;
        }
      }
    });

    Worker { thread }
  }
}
