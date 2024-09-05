use std::collections::HashMap;

use ecs_macros::all_tuples;

use crate::{components::Component, Id};

#[derive(Debug, Default)]
pub struct Entity(Id);

#[derive(Default)]
pub struct Entities {
  entities: Vec<Entity>,
  archetype_map: HashMap<Id, Id>,
  store: HashMap<Id, HashMap<Id, Vec<Box<dyn Component>>>>,
  free: Vec<Id>,
  top: Id,
}

impl Entities {
  pub fn create(&mut self, archetype: Id, components: Vec<Box<dyn Component>>) -> Id {
    let id = if let Some(id) = self.free.pop() {
      id
    } else {
      self.top += 1;
      self.top
    };

    self.entities.push(Entity(id));
    self.archetype_map.insert(id, archetype);

    let map = match self.store.get_mut(&archetype) {
      Some(map) => map,
      None => {
        self.store.insert(archetype, HashMap::new());
        self.store.get_mut(&archetype).unwrap()
      }
    };
    map.insert(id, components);

    id
  }

  pub fn get_entities_mut(&mut self, archetypes: Vec<Id>) -> Vec<&mut Vec<Box<dyn Component>>> {
    let mut res = Vec::new();
    for (type_, entities) in &mut self.store {
      if !archetypes.contains(type_) {
        continue;
      }
      for entity in entities.values_mut() {
        res.push(entity);
      }
    }

    res
  }
}

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
