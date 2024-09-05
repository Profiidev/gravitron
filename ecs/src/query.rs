use std::{collections::VecDeque, marker::PhantomData};

use crate::{
  components::Component, systems::SystemParam, world::UnsafeWorldCell, Id,
};

pub struct Query<'a, Q: QueryParam<'a>> {
  world: UnsafeWorldCell<'a>,
  marker: PhantomData<Q>
}

pub struct QueryIter<'a, Q: QueryParam<'a>> {
  entities: VecDeque<Q::Item>,
  marker: PhantomData<&'a Q>
}

impl<'a, Q: QueryParam<'a> + 'a> IntoIterator for Query<'a, Q> {
  type Item = Q::Item;
  type IntoIter = QueryIter<'a, Q>;

  fn into_iter(self) -> Self::IntoIter {
    let world = unsafe {
      self.world.world_mut()
    };

    let mut res = VecDeque::new();
    for entity in world.get_entities_mut(vec![1]) {
      res.push_back(Q::into_query(entity));
    }

    QueryIter {
      entities: res,
      marker: PhantomData
    }
  }
}

impl<'a, Q: QueryParam<'a>> Iterator for QueryIter<'a, Q> {
  type Item = Q::Item;

  fn next(&mut self) -> Option<Self::Item> {
    self.entities.pop_front()
  }
}

impl<'a, Q> SystemParam for Query<'a, Q>
where 
  for<'b> Q: QueryParam<'b>,
{
  type Item<'new> = Query<'new, Q>;

  fn get_param(world: UnsafeWorldCell<'_>) -> Self::Item<'_> {
    Query {
      world,
      marker: PhantomData
    }
  }
}

pub trait QueryParam<'a> {
  type Item: 'a;

  fn into_query(entity: &'a mut Vec<Box<dyn Component>>) -> Self::Item;
}

impl<'a, T1, T2> QueryParam<'a> for (T1, T2)
where
  T1: QueryParamItem<'a>,
  T2: QueryParamItem<'a>,
{
  type Item = (T1::Item, T2::Item);

  fn into_query(entity: &'a mut Vec<Box<dyn Component>>) -> Self::Item {
    let mut t1 = None;
    let mut t2 = None;
    for comp in entity {
      if comp.id() == T1::id() {
        t1 = Some(T1::into_param(comp));
      } else if comp.id() == T2::id() {
        t2 = Some(T2::into_param(comp));
      }
    }

    (t1.unwrap(), t2.unwrap())
  }
}

pub trait QueryParamItem<'a> {
  type Item: 'a;

  fn id() -> Id;
  fn into_param(comp: &'a mut Box<dyn Component>) -> Self::Item;
}

impl<'a, C: Component + 'static> QueryParamItem<'a> for &C {
  type Item = &'a C;

  fn id() -> Id {
    C::sid()
  }

  fn into_param(comp: &'a mut Box<dyn Component>) -> Self::Item {
    comp.downcast_ref().unwrap()
  }
}

impl<'a, C: Component + 'static> QueryParamItem<'a> for &mut C {
  type Item = &'a mut C;

  fn id() -> Id {
    C::sid()
  }

  fn into_param(comp: &'a mut Box<dyn Component>) -> Self::Item {
    comp.downcast_mut().unwrap()
  }
}
