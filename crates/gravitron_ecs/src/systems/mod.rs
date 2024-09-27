use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicU64, Ordering},
};

use log::trace;

use gravitron_ecs_macros::all_tuples;
use metadata::SystemMeta;

use crate::{world::UnsafeWorldCell, SystemId};

pub(crate) mod metadata;

static SYSTEM_ID: AtomicU64 = AtomicU64::new(0);

pub trait System: Send {
  fn run(&mut self, world: UnsafeWorldCell<'_>);
  fn get_meta(&self) -> &SystemMeta;
}

macro_rules! impl_system {
  ($($params:ident),*) => {
    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    impl<F: FnMut($($params),*) + Send, $($params : SystemParam),*> System for FunctionSystem<($($params ,)*), F>
    where
      for<'a, 'b> &'a mut F:
        FnMut($($params),*) +
        FnMut($(<$params as SystemParam>::Item<'b>),*)
    {
      fn run(&mut self, world: UnsafeWorldCell<'_>) {
        trace!("Executing System {}", self.id);
        #[allow(clippy::too_many_arguments)]
        fn call_inner<$($params),*>(
          mut f: impl FnMut($($params),*),
          $($params: $params),*
        ) {
          f($($params),*)
        }

        $(
          let $params = $params::get_param(world, self.id);
        )*

        call_inner(&mut self.f, $($params),*)
      }

      fn get_meta(&self) -> &SystemMeta {
        &self.meta
      }
    }

    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    impl<F: FnMut($($params),*) + Send, $($params : SystemParam),*> IntoSystem<($($params ,)*)> for F
    where
      for<'a, 'b> &'a mut F:
        FnMut($($params),*) +
        FnMut($(<$params as SystemParam>::Item<'b>),*)
    {
      type System = FunctionSystem<($($params ,)*), Self>;

      fn into_system(self) -> Self::System {
        #[allow(unused_mut)]
        let mut meta = SystemMeta::new();

        $(
          $params::check_metadata(&mut meta);
        )*

        let id = SYSTEM_ID.fetch_add(1, Ordering::SeqCst);

        FunctionSystem {
          f: self,
          meta,
          id,
          marker: Default::default()
        }
      }
    }
  };
}

all_tuples!(impl_system, 0, 16, F);

pub struct FunctionSystem<Input, F> {
  f: F,
  meta: SystemMeta,
  id: SystemId,
  marker: PhantomData<fn() -> Input>,
}

pub(crate) type StoredSystem = Box<dyn System>;

pub trait IntoSystem<Input> {
  type System: System;

  fn into_system(self) -> Self::System;
}

pub(crate) trait SystemParam {
  type Item<'new>;

  fn get_param(world: UnsafeWorldCell<'_>, id: SystemId) -> Self::Item<'_>;
  fn check_metadata(meta: &mut SystemMeta);
}

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

  fn get_param(world: UnsafeWorldCell<'_>, _: SystemId) -> Self::Item<'_> {
    let world = unsafe { world.world() };

    Res {
      value: world.get_resource().expect("Resource not found"),
    }
  }

  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_res::<T>(metadata::AccessType::Read);
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

  fn get_param(world: UnsafeWorldCell<'_>, _: SystemId) -> Self::Item<'_> {
    let world = unsafe { world.world_mut() };

    ResMut {
      value: world.get_resource_mut().expect("Resource not found"),
    }
  }

  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_res::<T>(metadata::AccessType::Read);
  }
}
