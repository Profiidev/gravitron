use downcast::{downcast, Any};

use crate::Id;

pub trait Component: Any {
  fn id(&self) -> Id;
  fn sid() -> Id
  where
    Self: Sized;
}

downcast!(dyn Component);

#[cfg(test)]
mod test {
  use crate::{self as gravitron_ecs, components::Component};
  use gravitron_ecs_macros::Component;

  #[derive(Component)]
  struct A {}

  #[derive(Component)]
  struct B {}

  #[test]
  fn check_id() {
    let a = A {};
    let b = B {};

    assert_eq!(A::sid(), a.id());
    assert_eq!(B::sid(), b.id());
  }

  #[test]
  fn downcast() {
    let a = A {};
    let box_a: Box<dyn Component> = Box::new(a);
    let a_cast = box_a.downcast::<A>().unwrap();
    assert_eq!(A::sid(), a_cast.id());
  }
}
