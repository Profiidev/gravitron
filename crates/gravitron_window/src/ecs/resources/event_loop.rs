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
  #[cfg(target_os = "linux")]
  wayland: bool,
}

impl EventLoop {
  pub(crate) fn init(config: WindowConfig) -> (Self, Window) {
    let (sender, receiver) = mpsc::channel();

    let ready_signal = Signal::new();
    let ready_signal_clone = Signal::clone_inner(&ready_signal);

    #[cfg(target_os = "linux")]
    let wayland_signal = Signal::new();
    #[cfg(target_os = "linux")]
    let wayland_signal_clone = Signal::clone_inner(&wayland_signal);

    let thread = thread::spawn(move || {
      WindowHandler::init(
        config,
        ready_signal,
        sender,
        #[cfg(target_os = "linux")]
        wayland_signal,
      )
      .expect("Failed to start EventLoop");
    });

    let window = ready_signal_clone.wait();
    #[cfg(target_os = "linux")]
    let wayland = wayland_signal_clone.wait();

    (
      Self {
        _thread: thread,
        receiver,
        events: Vec::new(),
        #[cfg(target_os = "linux")]
        wayland,
      },
      window,
    )
  }

  #[inline]
  pub(crate) fn update_events(&mut self) {
    self.events = self.receiver.try_iter().collect();
  }

  #[inline]
  pub fn events(&self) -> &[WindowEvent] {
    &self.events
  }

  #[cfg(target_os = "linux")]
  #[inline]
  pub fn wayland(&self) -> bool {
    self.wayland
  }
}
