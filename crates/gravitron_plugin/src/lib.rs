use app::{App, AppBuilder, Build, Finalize};

pub mod app;
pub mod config;
pub mod manager;
pub mod stages;

pub trait Plugin: 'static {
  fn build(&self, _builder: &mut AppBuilder<Build>) {}
  fn finalize(&self, _builder: &mut AppBuilder<Finalize>) {}
  fn cleanup(&self, _app: &mut App) {}
  fn name(&self) -> &str {
    std::any::type_name::<Self>()
  }
}
