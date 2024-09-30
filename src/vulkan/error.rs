use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueFamilyMissingError {
  #[error("No graphics queue family found")]
  Graphics,
  #[error("No compute queue family found")]
  Compute,
  #[error("No transfer queue family found")]
  Transfer,
}

#[derive(Error, Debug)]
pub enum RendererInitError {
  #[error("No surface formats found")]
  FormatMissing,
}
