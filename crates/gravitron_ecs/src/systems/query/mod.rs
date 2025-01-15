use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use gravitron_ecs_macros::all_tuples;
#[cfg(feature = "debug")]
use log::trace;

pub mod filter;

use crate::{
  components::{Component, UnsafeDowncast},
  storage::{ComponentBox, QueryResult, Row},
  systems::{
    metadata::{AccessType, QueryMeta, SystemMeta},
    SystemParam,
  },
  tick::Tick,
  world::UnsafeWorldCell,
  ComponentId, EntityId, SystemId,
};

pub struct Query<'a, Q: QueryParam> {
  world: UnsafeWorldCell<'a>,
  marker: PhantomData<Q>,
}

pub struct QueryIter<'a, Q: QueryParam> {
  archetypes: Vec<QueryResult<'a>>,
  archetype_index: usize,
  tick: Tick,
  marker: PhantomData<Q>,
}

impl<'a, Q: QueryParam + 'a> IntoIterator for Query<'a, Q> {
  type Item = Q::Item<'a>;
  type IntoIter = QueryIter<'a, Q>;

  fn into_iter(self) -> Self::IntoIter {
    let world = unsafe { self.world.world_mut() };
    let ids = Q::get_comp_ids();
    let tick = world.tick();

    #[cfg(feature = "debug")]
    trace!("Querying Entities {:?}", &ids);
    let archetypes = world.storage_mut().query_data(&ids);

    QueryIter {
      archetypes,
      archetype_index: 0,
      tick,
      marker: PhantomData,
    }
  }
}

impl<'a, Q: QueryParam> Iterator for QueryIter<'a, Q> {
  type Item = Q::Item<'a>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.archetype_index >= self.archetypes.len() {
      return None;
    }

    let QueryResult { rows, columns } = &mut self.archetypes[self.archetype_index];
    if rows.is_empty() {
      self.archetype_index += 1;
      return self.next();
    }

    let row = rows.pop()?;
    let item = Q::into_query(row, columns, self.tick);

    Some(item)
  }
}

impl<Q: QueryParam> SystemParam for Query<'_, Q> {
  type Item<'new> = Query<'new, Q>;

  #[inline]
  fn get_param(world: UnsafeWorldCell<'_>, _: SystemId) -> Self::Item<'_> {
    Query {
      world,
      marker: PhantomData,
    }
  }

  #[inline]
  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_query(Q::get_meta())
  }
}

pub trait QueryParam {
  type Item<'a>;

  fn into_query<'a>(entity: &'a mut Row, indices: &[usize], tick: Tick) -> Self::Item<'a>;
  fn get_meta() -> QueryMeta;
  fn get_comp_ids() -> Vec<ComponentId>;
}

macro_rules! impl_query_param {
  ($($params:ident),*) => {
    #[allow(unused_parens)]
    impl<$($params: QueryParamItem),*> QueryParam for ($($params),*) {
      type Item<'a> = (EntityId, $($params::Item<'a> ,)*);

      #[inline]
      #[allow(non_snake_case, unused_assignments)]
      fn into_query<'a>(entity: &'a mut Row, indices: &[usize], tick: Tick) -> Self::Item<'a> {
        let ptr = entity.comps.as_mut_ptr();
        params_enumerate!($($params)*, ptr, indices, tick);

        (entity.id, $($params),*)
      }

      #[inline]
      fn get_meta() -> QueryMeta {
        let mut meta = QueryMeta::new();

        $(
          $params::check_metadata(&mut meta);
        )*

        meta
      }

      #[inline]
      fn get_comp_ids() -> Vec<ComponentId> {
        vec![$($params::id()),*]
      }
    }
  };
}

macro_rules! params_enumerate {
  ($($params:ident)+, $ptr:ident, $indices:ident, $tick: ident) => {
    params_enumerate! {
      todo: [$($params)+],
      done: [],
      ptr: $ptr,
      indices: $indices,
      count: 0,
      tick: $tick,
    }
  };
  (
    todo: [$first:ident$($params:ident)*],
    done: [$($done:tt)*],
    ptr: $ptr:ident,
    indices: $indices:ident,
    count: $count:expr,
    tick: $tick:ident,
  ) => {
    params_enumerate! {
      todo: [$($params)*],
      done: [$($done)*[let $first = $first::into_param(unsafe { &mut *$ptr.add($indices[$count]) }, $tick);]],
      ptr: $ptr,
      indices: $indices,
      count: $count + 1,
      tick: $tick,
    }
  };
  (
    todo: [],
    done: [$([$($done:tt)+])*],
    ptr: $ptr:ident,
    indices: $indices:ident,
    count: $count:expr,
    tick: $tick:ident,
  ) => {
    $($($done)+)*
  };
}

all_tuples!(impl_query_param, 1, 16, F);

pub trait QueryParamItem {
  type Item<'a>;

  fn id() -> ComponentId;
  fn into_param(input: &mut ComponentBox, tick: Tick) -> Self::Item<'_>;
  fn check_metadata(meta: &mut QueryMeta);
}

impl<C: Component + 'static> QueryParamItem for &C {
  type Item<'a> = Ref<'a, C>;

  #[inline]
  fn id() -> ComponentId {
    C::sid()
  }

  #[inline]
  fn into_param(input: &mut ComponentBox, _: Tick) -> Self::Item<'_> {
    Ref(input, PhantomData)
  }

  #[inline]
  fn check_metadata(meta: &mut QueryMeta) {
    meta.add_comp::<C>(AccessType::Read);
  }
}

impl<C: Component + 'static> QueryParamItem for &mut C {
  type Item<'a> = Mut<'a, C>;

  #[inline]
  fn id() -> ComponentId {
    C::sid()
  }

  #[inline]
  fn into_param(input: &mut ComponentBox, tick: Tick) -> Self::Item<'_> {
    Mut(input, tick, PhantomData)
  }

  #[inline]
  fn check_metadata(meta: &mut QueryMeta) {
    meta.add_comp::<C>(AccessType::Write);
  }
}

pub struct Ref<'a, C>(&'a mut ComponentBox, PhantomData<C>);

impl<C: Component> Deref for Ref<'_, C> {
  type Target = C;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.comp.downcast_ref_unchecked() }
  }
}

pub struct Mut<'a, C>(&'a mut ComponentBox, Tick, PhantomData<C>);

impl<C: Component> Deref for Mut<'_, C> {
  type Target = C;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.comp.downcast_ref_unchecked() }
  }
}

impl<C: Component> DerefMut for Mut<'_, C> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.changed = self.1;
    unsafe { self.0.comp.downcast_mut_unchecked() }
  }
}
