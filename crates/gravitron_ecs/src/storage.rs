use rustc_hash::FxHashMap as HashMap;
use std::{
  marker::PhantomData,
  ptr,
  sync::atomic::{AtomicU64, Ordering},
};

#[cfg(feature = "debug")]
use log::trace;

use crate::{
  components::{Component, UnsafeDowncast},
  tick::Tick,
  ArchetypeId, ComponentId, EntityId, Id,
};

type Type = Vec<ComponentId>;
type ArchetypeMap<'a> = HashMap<ArchetypeId, ArchetypeRecord<'a>>;

pub struct Row {
  pub comps: Vec<ComponentBox>,
  pub id: EntityId,
  pub removed: HashMap<ComponentId, Tick>,
}

pub struct ComponentBox {
  pub comp: Box<dyn Component>,
  pub added: Tick,
  pub changed: (Tick, Tick),
}

struct ArchetypeEdge<'a> {
  add: UnsafeArchetypeCell<'a>,
  remove: UnsafeArchetypeCell<'a>,
}

struct Record<'a> {
  archetype: UnsafeArchetypeCell<'a>,
  row: usize,
}

struct ArchetypeRecord<'a> {
  column: usize,
  archetype: UnsafeArchetypeCell<'a>,
}

struct Archetype<'a> {
  id: ArchetypeId,
  r#type: Type,
  rows: Vec<Row>,
  edges: HashMap<ComponentId, ArchetypeEdge<'a>>,
}

#[derive(Clone, Copy)]
struct UnsafeArchetypeCell<'a>(*mut Archetype<'a>, PhantomData<&'a Archetype<'a>>);

unsafe impl Send for UnsafeArchetypeCell<'_> {}

unsafe impl Sync for UnsafeArchetypeCell<'_> {}

impl<'a> UnsafeArchetypeCell<'a> {
  fn new(archetype: &mut Archetype<'a>) -> Self {
    Self(ptr::from_mut(archetype), PhantomData)
  }

  fn null() -> Self {
    Self(ptr::null_mut(), PhantomData)
  }

  pub unsafe fn archetype_mut(&self) -> &'a mut Archetype<'a> {
    &mut *self.0
  }

  pub unsafe fn archetype(&self) -> &'a Archetype<'a> {
    &*self.0
  }
}

#[derive(Default)]
pub struct Storage<'a> {
  entity_index: HashMap<EntityId, Record<'a>>,
  archetype_index: HashMap<Type, Box<Archetype<'a>>>,
  component_index: HashMap<ComponentId, ArchetypeMap<'a>>,
  top_id: AtomicU64,
}

pub struct QueryResult<'a> {
  pub columns: Vec<usize>,
  pub rows: Vec<&'a mut Row>,
}

impl Storage<'_> {
  pub(crate) fn create_entity(&mut self, comps: Vec<Box<dyn Component>>, tick: Tick) -> EntityId {
    let id = Id(self.top_id.fetch_add(1, Ordering::Relaxed));

    self.create_entity_with_id(comps, id, tick);
    id
  }

  pub(crate) fn create_entity_with_id(
    &mut self,
    mut comps: Vec<Box<dyn Component>>,
    id: EntityId,
    tick: Tick,
  ) {
    #[cfg(feature = "debug")]
    trace!("Creating Entity {}", id);

    comps.sort_unstable_by_key(|c| c.id());
    let r#type = comps.iter().map(|c| c.id()).collect::<Type>();

    let archetype = if let Some(a) = self.archetype_index.get_mut(&r#type) {
      a
    } else {
      self.create_archetype(r#type.clone());
      self.archetype_index.get_mut(&r#type).unwrap()
    };

    let mut comp_box = Vec::new();
    for comp in comps {
      comp_box.push(ComponentBox {
        comp,
        added: tick,
        changed: (Tick::INVALID, Tick::INVALID),
      });
    }

    archetype.rows.push(Row {
      comps: comp_box,
      id,
      removed: Default::default(),
    });

    self.entity_index.insert(
      id,
      Record {
        archetype: UnsafeArchetypeCell::new(archetype),
        row: archetype.rows.len() - 1,
      },
    );
  }

  pub(crate) fn reserve_entity_id(&mut self) -> EntityId {
    #[cfg(feature = "debug")]
    trace!("Reserving EntityId");
    Id(self.top_id.fetch_add(1, Ordering::Relaxed))
  }

  pub fn remove_entity(&mut self, entity: EntityId) -> Option<()> {
    #[cfg(feature = "debug")]
    trace!("Removing Entity {}", entity);
    let record = self.entity_index.remove(&entity)?;
    let archetype = unsafe { record.archetype.archetype_mut() };

    archetype.rows.swap_remove(record.row);

    if let Some(swapped) = archetype.rows.get(record.row) {
      let swapped_record = self.entity_index.get_mut(&swapped.id).unwrap();
      swapped_record.row = record.row;
    }

    Some(())
  }

  fn create_archetype(&mut self, r#type: Type) {
    #[cfg(feature = "debug")]
    trace!("Creating Archetype {:?}", r#type);

    let archetype = Box::new(Archetype {
      id: Id(self.archetype_index.len() as u64),
      r#type: r#type.clone(),
      rows: Vec::new(),
      edges: HashMap::default(),
    });

    self.archetype_index.insert(r#type.clone(), archetype);
    let archetype = self.archetype_index.get_mut(&r#type).unwrap();
    let cell = UnsafeArchetypeCell::new(archetype);

    for (i, c) in r#type.iter().enumerate() {
      let ci = self.component_index.entry(*c).or_default();
      ci.insert(
        archetype.id,
        ArchetypeRecord {
          column: i,
          archetype: cell,
        },
      );
    }
  }

  pub fn get_comp<C: Component>(&mut self, entity: EntityId) -> Option<&mut C> {
    let record = self.entity_index.get(&entity)?;
    let archetype = unsafe { record.archetype.archetype_mut() };

    let archetypes = self.component_index.get(&C::sid())?;
    let a_record = archetypes.get(&archetype.id)?;

    let row = archetype.rows.get_mut(record.row)?;
    let component = row.comps.get_mut(a_record.column)?;

    Some(unsafe { component.comp.downcast_mut_unchecked() })
  }

  pub fn has_comp<C: Component>(&self, entity: EntityId) -> bool {
    let record = self.entity_index.get(&entity).unwrap();
    let archetype = unsafe { record.archetype.archetype() };
    archetype.r#type.contains(&C::sid())
  }

  pub fn add_comp(&mut self, entity: EntityId, comp: ComponentBox) {
    #[cfg(feature = "debug")]
    trace!("Adding Component {:?} to Entity {}", comp.comp.id(), entity);

    let record = self.entity_index.get_mut(&entity).unwrap();
    let from = unsafe { record.archetype.archetype_mut() };

    if from.r#type.contains(&comp.comp.id()) {
      return;
    }

    let to = if let Some(to) = from.edges.get(&comp.comp.id()) {
      unsafe { to.add.archetype_mut() }
    } else {
      let mut r#type = from.r#type.clone();
      r#type.push(comp.comp.id());
      r#type.sort_unstable();

      let to = if let Some(to) = self.archetype_index.get_mut(&r#type) {
        to
      } else {
        self.create_archetype(r#type.clone());
        self.archetype_index.get_mut(&r#type).unwrap()
      };

      from.edges.insert(
        comp.comp.id(),
        ArchetypeEdge {
          add: UnsafeArchetypeCell::new(to),
          remove: UnsafeArchetypeCell::null(),
        },
      );

      to
    };

    let record = self.entity_index.get_mut(&entity).unwrap();
    let new_comp = to.r#type.iter().position(|&c| c == comp.comp.id()).unwrap();

    let mut entity = from.rows.swap_remove(record.row);
    entity.comps.insert(new_comp, comp);
    to.rows.push(entity);

    let old_row = record.row;
    record.row = to.rows.len() - 1;
    record.archetype = UnsafeArchetypeCell::new(to);

    if let Some(swapped) = from.rows.get(old_row) {
      let swapped_record = self.entity_index.get_mut(&swapped.id).unwrap();
      swapped_record.row = old_row;
    }
  }

  pub fn remove_comp<C: Component>(&mut self, entity: EntityId, tick: Tick) -> Option<Box<C>> {
    #[cfg(feature = "debug")]
    trace!("Removing Component {:?} from Entity {}", C::sid(), entity);

    let record = self.entity_index.get_mut(&entity)?;
    let from = unsafe { record.archetype.archetype_mut() };

    if !from.r#type.contains(&C::sid()) {
      return None;
    }

    let to = if let Some(to) = from.edges.get(&C::sid()) {
      unsafe { to.remove.archetype_mut() }
    } else {
      let mut r#type = from.r#type.clone();
      r#type.retain(|t| t != &C::sid());
      r#type.sort_unstable();

      let to = if let Some(to) = self.archetype_index.get_mut(&r#type) {
        to
      } else {
        self.create_archetype(r#type.clone());
        self.archetype_index.get_mut(&r#type).unwrap()
      };

      from.edges.insert(
        C::sid(),
        ArchetypeEdge {
          remove: UnsafeArchetypeCell::new(to),
          add: UnsafeArchetypeCell::null(),
        },
      );

      to
    };

    let record = self.entity_index.get_mut(&entity)?;
    let removed_comp = from.r#type.iter().position(|&c| c == C::sid())?;

    let mut entity = from.rows.swap_remove(record.row);
    let component = entity.comps.remove(removed_comp);
    entity.removed.insert(C::sid(), tick);
    to.rows.push(entity);

    let old_row = record.row;
    record.row = to.rows.len() - 1;
    record.archetype = UnsafeArchetypeCell::new(to);

    if let Some(swapped) = from.rows.get(old_row) {
      let swapped_record = self.entity_index.get_mut(&swapped.id)?;
      swapped_record.row = old_row;
    }

    Some(unsafe { component.comp.downcast_unchecked() })
  }

  pub(crate) fn query_data<F>(
    &'_ mut self,
    comps: &[ComponentId],
    filter: F,
  ) -> Vec<QueryResult<'_>>
  where
    F: Fn(&[ComponentId]) -> bool,
  {
    if comps.is_empty() {
      return vec![];
    }

    let mut result = vec![];
    let Some(possible) = self.component_index.get(&comps[0]) else {
      return result;
    };

    for record in possible.values() {
      let archetype = unsafe { record.archetype.archetype_mut() };

      if comps.iter().all(|c| archetype.r#type.contains(c))
        && !archetype.rows.is_empty()
        && filter(&archetype.r#type)
      {
        let columns = comps
          .iter()
          .map(|c| {
            self
              .component_index
              .get(c)
              .unwrap()
              .get(&archetype.id)
              .unwrap()
              .column
          })
          .collect();

        let rows: Vec<&mut Row> = archetype.rows.iter_mut().collect();

        result.push(QueryResult { columns, rows });
      }
    }

    result
  }

  pub(crate) fn entity_by_id<F>(
    &mut self,
    entity: EntityId,
    comps: &[ComponentId],
    filter: F,
  ) -> Option<(&mut Row, Vec<usize>)>
  where
    F: Fn(&[ComponentId]) -> bool,
  {
    let record = self.entity_index.get_mut(&entity)?;
    let archetype = unsafe { record.archetype.archetype_mut() };

    if !comps.iter().all(|c| archetype.r#type.contains(c)) || !filter(&archetype.r#type) {
      return None;
    }

    let columns = comps
      .iter()
      .map(|c| {
        self
          .component_index
          .get(c)
          .unwrap()
          .get(&archetype.id)
          .unwrap()
          .column
      })
      .collect();

    Some((&mut archetype.rows[record.row], columns))
  }
}

impl ComponentBox {
  pub fn new<C: Component>(comp: C, tick: Tick) -> Self {
    ComponentBox {
      changed: (Tick::INVALID, Tick::INVALID),
      added: tick,
      comp: Box::new(comp),
    }
  }
}

#[cfg(test)]
mod test {
  use super::Storage;
  use crate::{self as gravitron_ecs, components::Component, storage::ComponentBox, tick::Tick};
  use gravitron_ecs_macros::Component;

  #[derive(Component)]
  struct A {}

  #[test]
  fn create_entity() {
    let mut storage = Storage::default();

    storage.create_entity(Vec::new(), Tick::default());
  }

  #[test]
  fn remove_entity() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new(), Tick::default());
    storage.remove_entity(id);
  }

  #[test]
  fn add_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new(), Tick::default());
    storage.add_comp(
      id,
      ComponentBox {
        comp: Box::new(A {}),
        added: Tick::default(),
        changed: (Tick::default(), Tick::default()),
      },
    );

    assert!(storage.has_comp::<A>(id));
  }

  #[test]
  fn remove_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new(), Tick::default());
    storage.add_comp(
      id,
      ComponentBox {
        comp: Box::new(A {}),
        added: Tick::default(),
        changed: (Tick::default(), Tick::default()),
      },
    );
    storage.remove_comp::<A>(id, Tick::default());

    assert!(!storage.has_comp::<A>(id));
  }

  #[test]
  fn has_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new(), Tick::default());
    storage.add_comp(
      id,
      ComponentBox {
        comp: Box::new(A {}),
        added: Tick::default(),
        changed: (Tick::default(), Tick::default()),
      },
    );

    assert!(storage.has_comp::<A>(id));
  }

  #[test]
  fn get_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new(), Tick::default());
    storage.add_comp(
      id,
      ComponentBox {
        comp: Box::new(A {}),
        added: Tick::default(),
        changed: (Tick::default(), Tick::default()),
      },
    );

    let comp = storage.get_comp::<A>(id).unwrap();
    assert!(comp.id() == A::sid());
  }
}
