#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum MainSystemStage {
  PreRender,
  RenderInit,
  RenderPrepare,
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
