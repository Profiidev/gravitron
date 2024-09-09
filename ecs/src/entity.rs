use ecs_macros::all_tuples;

use crate::components::Component;

pub trait IntoEntity {
  fn into_entity(self) -> Vec<Box<dyn Component>>;
}

macro_rules! impl_into_entity {
  ($($params:ident),*) => {
    #[allow(non_snake_case)]
    impl<$($params : Component + 'static),*> IntoEntity for ($($params ,)*) {
      fn into_entity(self) -> Vec<Box<dyn Component>> {
        let ($($params ,)*) = self;
        vec![$(Box::new($params)),*]
      }
    }
  };
}

all_tuples!(impl_into_entity, 1, 16, F);

