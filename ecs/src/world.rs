use std::{any::Any, marker::PhantomData, ptr};

use crate::{
  archetypes::Archetypes, components::Component, entity::{Entities, IntoEntity}, Id
};

#[derive(Default)]
pub struct World {
  archetypes: Archetypes,
  entities: Entities,
  resources: Vec<Box<dyn Any>>
}

impl World {
  pub fn new() -> Self {
    World::default()
  }

  pub fn add_entity(&mut self, entity: impl IntoEntity) -> Id {
    let components = entity.into_entity();
    let mut ids = Vec::new();
    for component in &components {
      ids.push(component.id());
    }

    let archetype = self.archetypes.get(ids);
    self.entities.create(archetype, components)
  }

  pub fn add_resource<R: 'static>(&mut self, res: R) {
    if self.get_resource::<R>().is_some() {
      return;
    }
    self.resources.push(Box::new(res));
  }

  pub fn get_resource<R: 'static>(&self) -> Option<&R> {
    for r in self.resources.iter() {
      if let Some(r) = r.downcast_ref::<R>() {
        return Some(r);
      }
    }

    None
  }

  pub fn get_resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
    for r in self.resources.iter_mut() {
      if let Some(r) = r.downcast_mut::<R>() {
        return Some(r);
      }
    }

    None
  }

  pub fn get_entities_mut(&mut self, archetypes: Vec<Id>) -> Vec<&mut Vec<Box<dyn Component>>> {
    self.entities.get_entities_mut(archetypes)
  }
}

#[derive(Clone, Copy)]
pub struct UnsafeWorldCell<'w>(*mut World, PhantomData<&'w World>);

unsafe impl Send for UnsafeWorldCell<'_> {}

unsafe impl Sync for UnsafeWorldCell<'_> {}

impl<'w> UnsafeWorldCell<'w> {
  pub fn new(world: &'w mut World) -> Self {
    Self(ptr::from_mut(world), PhantomData)
  }

  pub unsafe fn world_mut(&self) -> &'w mut World {
    &mut *self.0
  }

  pub unsafe fn world(&self) -> &'w World {
    &*self.0
  }
}

