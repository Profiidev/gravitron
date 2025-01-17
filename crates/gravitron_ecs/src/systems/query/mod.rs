use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use filter::QueryFilter;
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

pub struct Query<'a, Q: QueryParam, F: QueryFilter = ()> {
  world: UnsafeWorldCell<'a>,
  marker: PhantomData<(Q, F)>,
}

pub struct QueryIter<'a, Q: QueryParam, F: QueryFilter> {
  archetypes: Vec<QueryResult<'a>>,
  tick: Tick,
  marker: PhantomData<(Q, F)>,
}

impl<Q: QueryParam, F: QueryFilter> Query<'_, Q, F> {
  pub fn by_id(&mut self, entity: EntityId) -> Option<Q::Item<'_>> {
    let world = unsafe { self.world.world_mut() };
    let tick = world.tick();

    let storage = world.storage_mut();
    let ids = Q::get_comp_ids();

    let (row, columns) = storage.entity_by_id(entity, &ids, F::filter_archetype)?;

    if F::filter_entity(row, tick) {
      Some(Q::into_query(row, &columns, tick))
    } else {
      None
    }
  }
}

impl<'a, Q: QueryParam + 'a, F: QueryFilter> IntoIterator for Query<'a, Q, F> {
  type Item = Q::Item<'a>;
  type IntoIter = QueryIter<'a, Q, F>;

  fn into_iter(self) -> Self::IntoIter {
    let world = unsafe { self.world.world_mut() };
    let ids = Q::get_comp_ids();
    let tick = world.tick();

    #[cfg(feature = "debug")]
    trace!("Querying Entities {:?}", &ids);
    let archetypes = world.storage_mut().query_data(&ids, F::filter_archetype);

    QueryIter {
      archetypes,
      tick,
      marker: PhantomData,
    }
  }
}

impl<'a, Q: QueryParam, F: QueryFilter> Iterator for QueryIter<'a, Q, F> {
  type Item = Q::Item<'a>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let QueryResult { rows, columns } = self.archetypes.last_mut()?;

      while let Some(row) = rows.pop() {
        if F::filter_entity(row, self.tick) {
          return Some(Q::into_query(row, columns, self.tick));
        }
      }

      self.archetypes.pop();
    }
  }
}

impl<Q: QueryParam, F: QueryFilter> SystemParam for Query<'_, Q, F> {
  type Item<'new> = Query<'new, Q, F>;

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
    if self.0.changed.0 != self.1 {
      self.0.changed.1 = self.0.changed.0;
      self.0.changed.0 = self.1;
    }
    unsafe { self.0.comp.downcast_mut_unchecked() }
  }
}
