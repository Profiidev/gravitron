#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum MainSystemStage {
  PreRender,
  RenderInit,
  RenderRecording,
  RenderExecute,
  PostRender,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum InitSystemStage {
  PreInit,
  Init,
  PostInit,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum CleanupSystemStage {
  PreCleanup,
  Cleanup,
  PostCleanup,
}
