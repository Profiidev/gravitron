use std::any::Any;

use crate::ComponentId;

pub trait Component: Any {
  fn id(&self) -> ComponentId;
  fn sid() -> ComponentId
  where
    Self: Sized;
}

///from https://github.com/reem/rust-unsafe-any
/// # Safety
/// only use this if you are absolutely certain that the trait object is the struct you want
/// if it is another type it will result in undefined behavior
/// use [`std::any::Any`] instead if you are not certain
pub unsafe trait UnsafeDowncast {
  /// # Safety
  /// see trait
  unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
    &*data(self)
  }

  /// # Safety
  /// see trait
  unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
    &mut *data_mut(self)
  }

  /// # Safety
  /// see trait
  unsafe fn downcast_unchecked<T: Any>(self: Box<Self>) -> Box<T> {
    let raw: *mut Self = std::mem::transmute(self);
    std::mem::transmute(data_mut::<Self, T>(raw))
  }
}

///from https://github.com/reem/rust-traitobject
unsafe fn data<T: ?Sized, R>(val: *const T) -> *const R {
  val as *const R
}

///from https://github.com/reem/rust-traitobject
unsafe fn data_mut<T: ?Sized, R>(val: *mut T) -> *mut R {
  val as *mut R
}

unsafe impl UnsafeDowncast for dyn Component {}

#[cfg(test)]
mod test {
  use crate::{
    self as gravitron_ecs,
    components::{Component, UnsafeDowncast},
  };
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
    let a_cast = unsafe { box_a.downcast_ref_unchecked::<A>() };
    assert_eq!(A::sid(), a_cast.id());
  }
}
