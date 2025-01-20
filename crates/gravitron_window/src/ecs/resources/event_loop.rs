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
  events: Vec<WindowEvent>,
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
        events: Vec::new(),
      },
      window,
    )
  }

  pub(crate) fn update_events(&mut self) {
    self.events = self.receiver.try_iter().collect();
  }

  pub fn events(&self) -> &[WindowEvent] {
    &self.events
  }
}
