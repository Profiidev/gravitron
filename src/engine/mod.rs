use std::marker::PhantomData;

use manager::{ClientManager, Manager, ServerManager};

use crate::{config::EngineConfig, error::EngineBuildError, scene::Scene};

mod manager;
mod window;

pub struct Engine<S, M: Manager> {
  state: S,
  scene: Scene,
  manager: M,
}

pub struct EngineBuilder<S, M: Manager> {
  state: Option<S>,
  scene: Option<Scene>,
  config: Option<EngineConfig>,
  manager: PhantomData<M>
}

impl<S, M: Manager> Engine<S, M> {
  pub fn run(self) {
    self.manager.run();
  }
}

impl<S> Engine<S, ClientManager> {
  pub fn builder_client() -> EngineBuilder<S, ClientManager> {
    EngineBuilder {
      state: None,
      scene: None,
      config: None,
      manager: PhantomData
    }
  }
}

impl<S> Engine<S, ServerManager> {
  pub fn builder_server() -> EngineBuilder<S, ServerManager> {
    EngineBuilder {
      state: None,
      scene: None,
      config: None,
      manager: PhantomData
    }
  }
}

impl<S, M: Manager> EngineBuilder<S, M> {
  pub fn with_state(mut self, state: S) -> Self {
    self.state = Some(state);
    self
  }

  pub fn with_scene(mut self, scene: Scene) -> Self {
    self.scene = Some(scene);
    self
  }

  pub fn with_config(mut self, config: EngineConfig) -> Self {
    self.config = Some(config);
    self
  }

  pub fn build(self) -> Result<Engine<S, M>, EngineBuildError> {
    Ok(Engine {
      state: self.state.ok_or(EngineBuildError::StateMissing)?,
      scene: self.scene.ok_or(EngineBuildError::SceneMissing)?,
      manager: M::init(self.config.unwrap_or_default())
    })
  }
}