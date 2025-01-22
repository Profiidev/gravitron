use std::ops::Deref;

use gravitron_ecs::{
  commands::Commands,
  components::Component,
  systems::{
    metadata::SystemMeta,
    query::{
      filter::{Added, Changed, QueryFilter, With, Without},
      Query, QueryParam,
    },
    SystemParam,
  },
  world::UnsafeWorldCell,
  Id, SystemId,
};

use crate::components::{Children, Parent};

type RootQuery<'a, D> = Query<'a, &'a D, (Without<Parent>, With<D>)>;

/// # Important
/// If you use this you cant use the G component in any other query in the same system
/// and you cant use D or Children mutably
pub struct PropagationQuery<'a, D: Component, G: Component + PropagationUpdate<Data = D>> {
  data_query: Query<'a, &'a D>,
  global_data_query: Query<'a, &'a mut G>,
  nodes_query: Query<'a, &'a Children, With<D>>,
  root_query: Option<RootQuery<'a, D>>,
}

pub trait PropagationUpdate: Default {
  /// must be the normal type (D in PropagationQuery) for the propagated type (G in PropagationQuery) implementing this trait
  type Data: Component;

  fn update(&mut self, data: &Self::Data);
  fn copy(&self) -> Self;
}

impl<'a, D: Component, G: Component + PropagationUpdate<Data = D>> PropagationQuery<'a, D, G> {
  pub fn propagate(mut self, cmds: &mut Commands) {
    for (id, _) in self.root_query.take().unwrap() {
      self.propagate_recursive(id, G::default(), cmds);
    }
  }

  fn propagate_recursive(&mut self, entity: Id, mut state: G, cmds: &mut Commands) {
    if let Some((_, data)) = self.data_query.by_id(entity) {
      state.update(data.deref());
      if let Some((_, mut global)) = self.global_data_query.by_id(entity) {
        *global = state.copy();
      } else {
        cmds.add_comp(entity, state.copy());
      }

      if let Some((_, children)) = self.nodes_query.by_id(entity) {
        #[allow(clippy::unnecessary_to_owned)]
        for child in children.children().to_vec() {
          self.propagate_recursive(child, state.copy(), cmds);
        }
      }
    }
  }

  fn create(world: UnsafeWorldCell<'a>, id: SystemId) -> Self {
    Self {
      data_query: Query::get_param(world, id),
      global_data_query: Query::get_param(world, id),
      nodes_query: Query::get_param(world, id),
      root_query: Some(Query::get_param(world, id)),
    }
  }
}

impl<D: Component, G: Component + PropagationUpdate<Data = D>> SystemParam
  for PropagationQuery<'_, D, G>
{
  type Item<'new> = PropagationQuery<'new, D, G>;

  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_query(<&Children as QueryParam>::get_meta());
    meta.add_query(<&D as QueryParam>::get_meta());
    meta.add_query(<&mut G as QueryParam>::get_meta());
  }

  fn get_param(world: UnsafeWorldCell<'_>, id: SystemId) -> Self::Item<'_> {
    PropagationQuery::create(world, id)
  }
}

/// # Important
/// If you use this you cant use the G component in any other query in the same system
/// and you cant use D, Parent or Children mutably
pub struct UpdatePropagationQuery<'a, D: Component, G: Component + PropagationUpdate<Data = D>> {
  data_query: Query<'a, &'a D>,
  global_data_query: Query<'a, &'a mut G>,
  nodes_query: Query<'a, &'a Children, With<D>>,
  nodes_parent_query: Query<'a, &'a Parent, With<D>>,
  changed_query: Option<Query<'a, &'a D, Changed<D>>>,
  added_query: Option<Query<'a, &'a D, Added<D>>>,
}

impl<'a, D: Component, G: Component + PropagationUpdate<Data = D>>
  UpdatePropagationQuery<'a, D, G>
{
  pub fn propagate(mut self, cmds: &mut Commands) {
    let query = self.added_query.take().unwrap();
    self.iterate_over_query(query, cmds);
    let query = self.changed_query.take().unwrap();
    self.iterate_over_query(query, cmds);
  }

  fn iterate_over_query<F: QueryFilter>(
    &mut self,
    query: Query<'a, &'a D, F>,
    cmds: &mut Commands,
  ) {
    let mut seen = Vec::new();

    for (id, _) in query {
      if seen.contains(&id) {
        continue;
      }

      let global = if let Some((_, parent)) = self.nodes_parent_query.by_id(id) {
        if let Some((_, global)) = self.global_data_query.by_id(parent.parent()) {
          global.copy()
        } else {
          continue;
        }
      } else {
        G::default()
      };

      self.propagate_recursive(id, global, cmds, &mut seen);
    }
  }

  fn propagate_recursive(
    &mut self,
    entity: Id,
    mut state: G,
    cmds: &mut Commands,
    seen: &mut Vec<Id>,
  ) {
    if let Some((_, data)) = self.data_query.by_id(entity) {
      state.update(data.deref());
      if let Some((_, mut global)) = self.global_data_query.by_id(entity) {
        *global = state.copy();
      } else {
        cmds.add_comp(entity, state.copy());
      }

      if let Some((_, children)) = self.nodes_query.by_id(entity) {
        #[allow(clippy::unnecessary_to_owned)]
        for child in children.children().to_vec() {
          if !seen.contains(&child) {
            seen.push(child);
          }

          self.propagate_recursive(child, state.copy(), cmds, seen);
        }
      }
    }
  }

  fn create(world: UnsafeWorldCell<'a>, id: SystemId) -> Self {
    Self {
      data_query: Query::get_param(world, id),
      global_data_query: Query::get_param(world, id),
      nodes_query: Query::get_param(world, id),
      nodes_parent_query: Query::get_param(world, id),
      changed_query: Some(Query::get_param(world, id)),
      added_query: Some(Query::get_param(world, id)),
    }
  }
}

impl<D: Component, G: Component + PropagationUpdate<Data = D>> SystemParam
  for UpdatePropagationQuery<'_, D, G>
{
  type Item<'new> = UpdatePropagationQuery<'new, D, G>;

  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_query(<&Children as QueryParam>::get_meta());
    meta.add_query(<&Parent as QueryParam>::get_meta());
    meta.add_query(<&D as QueryParam>::get_meta());
    meta.add_query(<&mut G as QueryParam>::get_meta());
  }

  fn get_param(world: UnsafeWorldCell<'_>, id: SystemId) -> Self::Item<'_> {
    UpdatePropagationQuery::create(world, id)
  }
}
