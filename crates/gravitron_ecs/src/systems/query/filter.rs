use std::marker::PhantomData;

use gravitron_ecs_macros::all_tuples;

use crate::{components::Component, storage::Row, tick::Tick, ComponentId};

pub trait QueryFilter {
  fn filter_archetype(r#type: &[ComponentId]) -> bool;
  fn filter_entity(entity: &Row, tick: Tick) -> bool;
}

pub trait QueryFilterParam {
  fn filter_archetype(r#type: &[ComponentId]) -> bool;
  fn filter_entity(entity: &Row, tick: Tick) -> bool;
}

impl QueryFilter for () {
  #[inline]
  fn filter_archetype(_: &[ComponentId]) -> bool {
    true
  }

  #[inline]
  fn filter_entity(_: &Row, _: Tick) -> bool {
    true
  }
}

macro_rules! impl_query_filter {
  ($($params:ident),*) => {
    #[allow(unused_parens)]
    impl<$($params: QueryFilterParam),*> QueryFilter for ($($params),*) {
      fn filter_archetype(r#type: &[ComponentId]) -> bool {
        $(
          $params::filter_archetype(r#type)
        )&&*
      }

      fn filter_entity(entity: &Row, tick: Tick) -> bool {
        $(
          $params::filter_entity(entity, tick)
        )&&*
      }
    }
  };
}

all_tuples!(impl_query_filter, 1, 16, F);

pub struct With<C: Component>(PhantomData<C>);

impl<C: Component> QueryFilterParam for With<C> {
  fn filter_archetype(r#type: &[ComponentId]) -> bool {
    r#type.contains(&C::sid())
  }

  fn filter_entity(_: &Row, _: Tick) -> bool {
    true
  }
}

pub struct Without<C: Component>(PhantomData<C>);

impl<C: Component> QueryFilterParam for Without<C> {
  fn filter_archetype(r#type: &[ComponentId]) -> bool {
    !r#type.contains(&C::sid())
  }

  fn filter_entity(_: &Row, _: Tick) -> bool {
    true
  }
}

pub struct Added<C: Component>(PhantomData<C>);

impl<C: Component> QueryFilterParam for Added<C> {
  fn filter_archetype(_: &[ComponentId]) -> bool {
    true
  }

  fn filter_entity(entity: &Row, tick: Tick) -> bool {
    entity
      .comps
      .iter()
      .any(|c| c.comp.id() == C::sid() && c.added == tick.last())
  }
}

pub struct Changed<C: Component>(PhantomData<C>);

impl<C: Component> QueryFilterParam for Changed<C> {
  fn filter_archetype(_: &[ComponentId]) -> bool {
    true
  }

  fn filter_entity(entity: &Row, tick: Tick) -> bool {
    entity.comps.iter().any(|c| {
      c.comp.id() == C::sid() && (c.changed.0 == tick.last() || c.changed.1 == tick.last())
    })
  }
}

pub struct Removed<C: Component>(PhantomData<C>);

impl<C: Component> QueryFilterParam for Removed<C> {
  fn filter_archetype(_: &[ComponentId]) -> bool {
    true
  }

  fn filter_entity(entity: &Row, tick: Tick) -> bool {
    entity.removed.get(&C::sid()) == Some(&tick.last())
  }
}

pub struct Or<F1: QueryFilter, F2: QueryFilter>(PhantomData<(F1, F2)>);

impl<F1: QueryFilter, F2: QueryFilter> QueryFilterParam for Or<F1, F2> {
  fn filter_archetype(r#type: &[ComponentId]) -> bool {
    F1::filter_archetype(r#type) || F2::filter_archetype(r#type)
  }

  fn filter_entity(entity: &Row, tick: Tick) -> bool {
    F1::filter_entity(entity, tick) || F2::filter_entity(entity, tick)
  }
}
