use gxhash::HashMap;
use std::{
  marker::PhantomData,
  ptr,
  sync::atomic::{AtomicU64, Ordering},
};

#[cfg(feature = "debug")]
use log::trace;

use crate::{components::Component, tick::Tick, ArchetypeId, ComponentId, EntityId, Id};

type Type = Vec<ComponentId>;
type ArchetypeMap<'a> = HashMap<ArchetypeId, ArchetypeRecord<'a>>;

pub struct Row {
  pub comps: Vec<ComponentBox>,
  pub id: EntityId,
}

pub struct ComponentBox {
  pub comp: Box<dyn Component>,
  pub added: Tick,
  pub changed: Tick,
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
  type_: Type,
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
  pub fn create_entity(&mut self, comps: Vec<Box<dyn Component>>, tick: Tick) -> EntityId {
    let id = Id(self.top_id.fetch_add(1, Ordering::Relaxed));

    self.create_entity_with_id(comps, id, tick);
    id
  }

  pub fn create_entity_with_id(
    &mut self,
    mut comps: Vec<Box<dyn Component>>,
    id: EntityId,
    tick: Tick,
  ) {
    #[cfg(feature = "debug")]
    trace!("Creating Entity {}", id);

    comps.sort_unstable_by_key(|c| c.id());
    let type_ = comps.iter().map(|c| c.id()).collect::<Type>();

    let archetype = if let Some(a) = self.archetype_index.get_mut(&type_) {
      a
    } else {
      self.create_archetype(type_.clone());
      self.archetype_index.get_mut(&type_).unwrap()
    };

    let mut comp_box = Vec::new();
    for comp in comps {
      comp_box.push(ComponentBox {
        comp,
        added: tick,
        changed: Tick::default(),
      });
    }

    archetype.rows.push(Row {
      comps: comp_box,
      id,
    });

    self.entity_index.insert(
      id,
      Record {
        archetype: UnsafeArchetypeCell::new(archetype),
        row: archetype.rows.len() - 1,
      },
    );
  }

  pub fn reserve_entity_id(&mut self) -> EntityId {
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

  pub fn create_archetype(&mut self, type_: Type) {
    #[cfg(feature = "debug")]
    trace!("Creating Archetype {:?}", type_);

    let archetype = Box::new(Archetype {
      id: Id(self.archetype_index.len() as u64),
      type_: type_.clone(),
      rows: Vec::new(),
      edges: HashMap::default(),
    });

    self.archetype_index.insert(type_.clone(), archetype);
    let archetype = self.archetype_index.get_mut(&type_).unwrap();
    let cell = UnsafeArchetypeCell::new(archetype);

    for (i, c) in type_.iter().enumerate() {
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

  #[allow(unused)]
  pub fn get_comp(&self, entity: EntityId, comp: ComponentId) -> Option<&dyn Component> {
    let record = self.entity_index.get(&entity)?;
    let archetype = unsafe { record.archetype.archetype() };

    let archetypes = self.component_index.get(&comp)?;
    let a_record = archetypes.get(&archetype.id)?;

    let row = archetype.rows.get(record.row)?;
    let component = row.comps.get(a_record.column)?;

    Some(&*component.comp)
  }

  #[allow(unused)]
  pub fn has_comp(&self, entity: EntityId, comp: ComponentId) -> bool {
    let record = self.entity_index.get(&entity).unwrap();
    let archetype = unsafe { record.archetype.archetype() };
    archetype.type_.contains(&comp)
  }

  pub fn add_comp(&mut self, entity: EntityId, comp: ComponentBox) {
    #[cfg(feature = "debug")]
    trace!("Adding Component {:?} to Entity {}", comp.comp.id(), entity);

    let record = self.entity_index.get_mut(&entity).unwrap();
    let from = unsafe { record.archetype.archetype_mut() };

    if from.type_.contains(&comp.comp.id()) {
      return;
    }

    let to = if let Some(to) = from.edges.get(&comp.comp.id()) {
      unsafe { to.add.archetype_mut() }
    } else {
      let mut type_ = from.type_.clone();
      type_.push(comp.comp.id());
      type_.sort_unstable();

      let to = if let Some(to) = self.archetype_index.get_mut(&type_) {
        to
      } else {
        self.create_archetype(type_.clone());
        self.archetype_index.get_mut(&type_).unwrap()
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
    let new_comp = to.type_.iter().position(|&c| c == comp.comp.id()).unwrap();

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

  pub fn remove_comp(&mut self, entity: EntityId, comp: ComponentId) {
    #[cfg(feature = "debug")]
    trace!("Removing Component {:?} from Entity {}", comp, entity);

    let record = self.entity_index.get_mut(&entity).unwrap();
    let from = unsafe { record.archetype.archetype_mut() };

    if !from.type_.contains(&comp) {
      return;
    }

    let to = if let Some(to) = from.edges.get(&comp) {
      unsafe { to.remove.archetype_mut() }
    } else {
      let mut type_ = from.type_.clone();
      type_.retain(|t| t != &comp);
      type_.sort_unstable();

      let to = if let Some(to) = self.archetype_index.get_mut(&type_) {
        to
      } else {
        self.create_archetype(type_.clone());
        self.archetype_index.get_mut(&type_).unwrap()
      };

      from.edges.insert(
        comp,
        ArchetypeEdge {
          remove: UnsafeArchetypeCell::new(to),
          add: UnsafeArchetypeCell::null(),
        },
      );

      to
    };

    let record = self.entity_index.get_mut(&entity).unwrap();
    let removed_comp = from.type_.iter().position(|&c| c == comp).unwrap();

    let mut entity = from.rows.swap_remove(record.row);
    entity.comps.remove(removed_comp);
    to.rows.push(entity);

    let old_row = record.row;
    record.row = to.rows.len() - 1;
    record.archetype = UnsafeArchetypeCell::new(to);

    if let Some(swapped) = from.rows.get(old_row) {
      let swapped_record = self.entity_index.get_mut(&swapped.id).unwrap();
      swapped_record.row = old_row;
    }
  }

  pub(crate) fn query_data(&mut self, comps: &[ComponentId]) -> Vec<QueryResult> {
    if comps.is_empty() {
      return vec![];
    }

    let mut result = vec![];
    let possible = self.component_index.get(&comps[0]).unwrap();
    for record in possible.values() {
      let archetype = unsafe { record.archetype.archetype_mut() };

      if comps.iter().all(|c| archetype.type_.contains(c)) && !archetype.rows.is_empty() {
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
        changed: Tick::default(),
      },
    );

    assert!(storage.has_comp(id, A::sid()));
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
        changed: Tick::default(),
      },
    );
    storage.remove_comp(id, A::sid());

    assert!(!storage.has_comp(id, A::sid()));
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
        changed: Tick::default(),
      },
    );

    assert!(storage.has_comp(id, A::sid()));
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
        changed: Tick::default(),
      },
    );

    let comp = storage.get_comp(id, A::sid()).unwrap();
    assert!(comp.id() == A::sid());
  }
}
