use downcast::{downcast, Any};

use crate::Id;

pub trait Component: Any {
  fn id(&self) -> Id;
  fn sid() -> Id where Self: Sized;
}

downcast!(dyn Component);

