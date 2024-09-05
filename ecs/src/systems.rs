use std::{
  any::{Any, TypeId},
  cell::UnsafeCell,
  collections::HashMap,
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use ecs_macros::all_tuples;

use crate::world::UnsafeWorldCell;

pub trait System {
  fn run(&mut self, world: UnsafeWorldCell<'_>);
}

macro_rules! impl_system {
  ($($params:ident),*) => {
    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    impl<F: FnMut($($params),*), $($params : SystemParam),*> System for FunctionSystem<($($params ,)*), F>
    where
      for<'a, 'b> &'a mut F:
        FnMut($($params),*) +
        FnMut($(<$params as SystemParam>::Item<'b>),*)
    {
      fn run(&mut self, world: UnsafeWorldCell<'_>) {
        #[allow(clippy::too_many_arguments)]
        fn call_inner<$($params),*>(
          mut f: impl FnMut($($params),*),
          $($params: $params),*
        ) {
          f($($params),*)
        }

        $(
          let $params = $params::get_param(world);
        )*

        call_inner(&mut self.f, $($params),*)
      }
    }

    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    impl<F: FnMut($($params),*), $($params : SystemParam),*> IntoSystem<($($params ,)*)> for F
    where
      for<'a, 'b> &'a mut F:
        FnMut($($params),*) +
        FnMut($(<$params as SystemParam>::Item<'b>),*)
    {
      type System = FunctionSystem<($($params ,)*), Self>;

      fn into_system(self) -> Self::System {
        FunctionSystem {
          f: self,
          marker: Default::default()
        }
      }
    }
  };
}

all_tuples!(impl_system, 0, 16, F);

pub struct FunctionSystem<Input, F> {
  f: F,
  marker: PhantomData<fn() -> Input>,
}

pub type StoredSystem = Box<dyn System>;

pub trait IntoSystem<Input> {
  type System: System;

  fn into_system(self) -> Self::System;
}

pub trait SystemParam {
  type Item<'new>;

  fn get_param(world: UnsafeWorldCell<'_>) -> Self::Item<'_>;
}

pub type TypeMap = HashMap<TypeId, UnsafeCell<Box<dyn Any>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
  Read,
  Write,
}

pub type AccessMap = HashMap<TypeId, Access>;

pub struct Res<'a, T: 'static> {
  value: &'a T,
}

impl<T: 'static> Deref for Res<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.value
  }
}

impl<'res, T: 'static> SystemParam for Res<'res, T> {
  type Item<'new> = Res<'new, T>;

  fn get_param(world: UnsafeWorldCell<'_>) -> Self::Item<'_> {
    let world = unsafe {
      world.world()
    };

    Res {
      value: world.get_resource().unwrap()
    }
  }
}

pub struct ResMut<'a, T: 'static> {
  value: &'a mut T,
}

impl<T: 'static> Deref for ResMut<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.value
  }
}

impl<T: 'static> DerefMut for ResMut<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.value
  }
}

impl<'res, T: 'static> SystemParam for ResMut<'res, T> {
  type Item<'new> = ResMut<'new, T>;

  fn get_param(world: UnsafeWorldCell<'_>) -> Self::Item<'_> {
    let world = unsafe {
      world.world_mut()
    };

    ResMut {
      value: world.get_resource_mut().unwrap()
    }
  }
}
