use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineBuildError {
  #[error("State is missing")]
  StateMissing,
  #[error("Scene is missing")]
  SceneMissing,
}
