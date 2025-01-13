use std::marker::PhantomData;

use gravitron_ecs_macros::all_tuples;
#[cfg(feature = "debug")]
use log::trace;

mod filter;

use crate::{
  components::Component, storage::QueryResult, systems::{
    metadata::{AccessType, QueryMeta, SystemMeta},
    SystemParam,
  }, world::UnsafeWorldCell, ComponentId, EntityId, SystemId
};

pub struct Query<'a, Q: QueryParam> {
  world: UnsafeWorldCell<'a>,
  marker: PhantomData<Q>,
}


pub struct QueryIter<'a, Q: QueryParam> {
  archetypes: Vec<QueryResult<'a>>,
  archetype_index: usize,
  marker: PhantomData<Q>,
}

impl<'a, Q: QueryParam + 'a> IntoIterator for Query<'a, Q> {
  type Item = Q::Item<'a>;
  type IntoIter = QueryIter<'a, Q>;

  fn into_iter(self) -> Self::IntoIter {
    let world = unsafe { self.world.world_mut() };
    let ids = Q::get_comp_ids();
    let archetypes = world.storage_mut().query_data(&ids);

    #[cfg(feature = "debug")]
    trace!("Querying Entities {:?}", &ids);

    QueryIter {
      archetypes,
      archetype_index: 0,
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

    let QueryResult {
      ids,
      comps,
      columns,
    } = &mut self.archetypes[self.archetype_index];
    if ids.is_empty() {
      self.archetype_index += 1;
      return self.next();
    }

    let entity_id = ids.pop()?;
    let comps = comps.pop()?;
    let item = Q::into_query((entity_id, comps), columns);

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

  fn into_query<'a>(
    entity: (EntityId, &'a mut Vec<Box<dyn Component>>),
    indices: &[usize],
  ) -> Self::Item<'a>;
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
      fn into_query<'a>(entity: (EntityId, &'a mut Vec<Box<dyn Component>>), indices: &[usize]) -> Self::Item<'a> {
        let ptr = entity.1.as_mut_ptr();
        let mut i = 0;
        $(
          let $params = $params::into_param(unsafe { &mut **ptr.add(indices[i]) });
          i += 1;
        )*

        (entity.0, $($params),*)
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

all_tuples!(impl_query_param, 1, 16, F);

pub trait QueryParamItem {
  type Item<'a>;

  fn id() -> ComponentId;
  fn into_param(input: &mut dyn Component) -> Self::Item<'_>;
  fn check_metadata(meta: &mut QueryMeta);
}

impl<C: Component + 'static> QueryParamItem for &C {
  type Item<'a> = &'a C;

  #[inline]
  fn id() -> ComponentId {
    C::sid()
  }

  #[inline]
  fn into_param(input: &mut dyn Component) -> Self::Item<'_> {
    input.downcast_ref().unwrap()
  }

  #[inline]
  fn check_metadata(meta: &mut QueryMeta) {
    meta.add_comp::<C>(AccessType::Read);
  }
}

impl<C: Component + 'static> QueryParamItem for &mut C {
  type Item<'a> = &'a mut C;

  #[inline]
  fn id() -> ComponentId {
    C::sid()
  }

  #[inline]
  fn into_param(input: &mut dyn Component) -> Self::Item<'_> {
    input.downcast_mut().unwrap()
  }

  #[inline]
  fn check_metadata(meta: &mut QueryMeta) {
    meta.add_comp::<C>(AccessType::Write);
  }
}
