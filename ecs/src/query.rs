use std::{collections::VecDeque, marker::PhantomData};

use ecs_macros::all_tuples;

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
    for entity in world.get_entities_mut(vec![0, 1]) {
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

macro_rules! impl_query_param {
  ($one:ident) => {
    impl<'a, $one: QueryParamItem<'a>> QueryParam<'a> for $one {
      type Item = $one::Item;

      #[allow(non_snake_case)]
      fn into_query(entity: &'a mut Vec<Box<dyn Component>>) -> Self::Item {
        let mut $one = None;

        for comp in entity {
          if comp.id() == $one::id() {
            $one = Some($one::into_param(comp));
          }
        }

        $one.unwrap()
      }
    }

    impl_query_param!($one,);
  };
  ($first:ident, $($params:ident),*) => {
    impl<'a, $first: QueryParamItem<'a>, $($params: QueryParamItem<'a>),*> QueryParam<'a> for ($first, $($params),*) {
      type Item = ($first::Item, $($params::Item ,)*);

      #[allow(non_snake_case)]
      fn into_query(entity: &'a mut Vec<Box<dyn Component>>) -> Self::Item {
        let mut $first = None;
        $(
          let mut $params = None;
        )*

        for comp in entity {
          if comp.id() == $first::id() {
            $first = Some($first::into_param(comp));
          }
          $(
            else if comp.id() == $params::id() {
              $params = Some($params::into_param(comp));
            }
          )*
        }

        ($first.unwrap(), $($params.unwrap()),*)
      }
    }
  };
}

all_tuples!(impl_query_param, 1, 16, F);

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
