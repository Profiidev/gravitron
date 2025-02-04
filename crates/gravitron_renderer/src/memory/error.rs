use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
  #[error("Buffer not found")]
  NotFound,
  #[error("Buffer reallocate error")]
  Reallocate,
}
