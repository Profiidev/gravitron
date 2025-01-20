use std::{
  sync::mpsc::{self, Receiver},
  thread::{self, JoinHandle},
};

use gravitron_plugin::config::window::WindowConfig;
use gravitron_utils::thread::Signal;
use winit::{event::WindowEvent, window::Window};

use crate::window::WindowHandler;

pub struct EventLoop {
  _thread: JoinHandle<()>,
  receiver: Receiver<WindowEvent>,
}

impl EventLoop {
  pub(crate) fn init(config: WindowConfig) -> (Self, Window) {
    let (sender, receiver) = mpsc::channel();

    let ready_signal = Signal::new();
    let ready_signal_clone = Signal::clone_inner(&ready_signal);

    let thread = thread::spawn(move || {
      WindowHandler::init(config, ready_signal, sender).expect("Failed to start EventLoop");
    });

    let window = ready_signal_clone.wait();

    (
      Self {
        _thread: thread,
        receiver,
      },
      window,
    )
  }

  pub(crate) fn get_events(&self) -> Vec<WindowEvent> {
    self.receiver.try_iter().collect()
  }
}
