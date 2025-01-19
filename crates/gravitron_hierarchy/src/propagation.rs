use gravitron_ecs::{
  commands::Commands,
  components::Component,
  systems::{
    metadata::SystemMeta,
    query::{
      filter::{With, Without},
      Mut, Query, QueryParam, Ref,
    },
    SystemParam,
  },
  world::UnsafeWorldCell,
  Id, SystemId,
};

use crate::components::{Children, Parent};

type RootQuery<'a, D> = Query<'a, &'a Children, (Without<Parent>, With<D>)>;

/// # Important
/// If you use this you cant use the G component in any other query in the same system
pub struct PropagationQuery<'a, D: Component, G: Component> {
  data_query: Query<'a, &'a D>,
  global_data_query: Query<'a, &'a mut G>,
  nodes_query: Query<'a, &'a Children, With<D>>,
  root_query: Option<RootQuery<'a, D>>,
}

impl<'a, D: Component, G: Component> PropagationQuery<'a, D, G> {
  pub fn propagate<U, C, US, S: Clone>(
    mut self,
    update_global: U,
    create_global: C,
    update_state: US,
    initial_state: S,
    cmds: &mut Commands,
  ) where
    U: Fn(Mut<'_, G>, &S),
    C: Fn(&S) -> G,
    US: Fn(Ref<'_, D>, &mut S),
  {
    for (id, children) in self.root_query.take().unwrap() {
      let children = children.children().to_vec();

      self.propagate_recursive(
        (id, children),
        &update_global,
        &create_global,
        &update_state,
        initial_state.clone(),
        cmds,
      );
    }
  }

  fn propagate_recursive<U, C, US, S: Clone>(
    &mut self,
    entity: (Id, Vec<Id>),
    update_global: &U,
    create_global: &C,
    update_state: &US,
    mut state: S,
    cmds: &mut Commands,
  ) where
    U: Fn(Mut<'_, G>, &S),
    C: Fn(&S) -> G,
    US: Fn(Ref<'_, D>, &mut S),
  {
    if let Some((_, data)) = self.data_query.by_id(entity.0) {
      update_state(data, &mut state);
      if let Some((_, global)) = self.global_data_query.by_id(entity.0) {
        update_global(global, &state);
      } else {
        cmds.add_comp(entity.0, create_global(&state));
      }

      for child in entity.1 {
        if let Some((id, children)) = self.nodes_query.by_id(child) {
          let children: Vec<Id> = children.children().to_vec();

          self.propagate_recursive(
            (id, children),
            update_global,
            create_global,
            update_state,
            state.clone(),
            cmds,
          );
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

impl<D: Component, G: Component> SystemParam for PropagationQuery<'_, D, G> {
  type Item<'new> = PropagationQuery<'new, D, G>;

  fn check_metadata(meta: &mut SystemMeta) {
    meta.add_query(<&Children as QueryParam>::get_meta());
    meta.add_query(<&Children as QueryParam>::get_meta());
    meta.add_query(<&D as QueryParam>::get_meta());
    meta.add_query(<&mut G as QueryParam>::get_meta());
  }

  fn get_param(world: UnsafeWorldCell<'_>, id: SystemId) -> Self::Item<'_> {
    PropagationQuery::create(world, id)
  }
}
