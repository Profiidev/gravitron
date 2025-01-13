use gxhash::HashMap;
use std::{
  collections::VecDeque,
  marker::PhantomData,
  ptr,
  sync::atomic::{AtomicU64, Ordering},
};

#[cfg(feature = "debug")]
use log::trace;

use crate::{components::Component, ArchetypeId, ComponentId, EntityId, Id};

type Type = Vec<ComponentId>;
type ArchetypeMap<'a> = HashMap<ArchetypeId, ArchetypeRecord<'a>>;
type Row = Vec<Box<dyn Component>>;

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
  entity_ids: Vec<EntityId>,
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
  archetype_index: HashMap<Type, Archetype<'a>>,
  component_index: HashMap<ComponentId, ArchetypeMap<'a>>,
  top_id: AtomicU64,
}

pub struct QueryResult<'a> {
  pub ids: Vec<EntityId>,
  pub comps: Vec<&'a mut Vec<Box<dyn Component>>>,
  pub columns: Vec<usize>,
}

impl Storage<'_> {
  pub fn create_entity(&mut self, comps: Vec<Box<dyn Component>>) -> EntityId {
    let id = Id(self.top_id.fetch_add(1, Ordering::Relaxed));

    self.create_entity_with_id(comps, id);
    id
  }

  pub fn create_entity_with_id(&mut self, mut comps: Vec<Box<dyn Component>>, id: EntityId) {
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

    archetype.entity_ids.push(id);
    archetype.rows.push(comps);

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

    archetype.entity_ids.swap_remove(record.row);
    archetype.rows.swap_remove(record.row);

    if let Some(swapped) = archetype.entity_ids.get(record.row) {
      let swapped_record = self.entity_index.get_mut(swapped).unwrap();
      swapped_record.row = record.row;
    }

    Some(())
  }

  pub fn create_archetype(&mut self, type_: Type) {
    #[cfg(feature = "debug")]
    trace!("Creating Archetype {:?}", type_);

    let archetype = Archetype {
      id: Id(self.archetype_index.len() as u64),
      type_: type_.clone(),
      entity_ids: Vec::new(),
      rows: Vec::new(),
      edges: HashMap::default(),
    };

    self.archetype_index.insert(type_.clone(), archetype);
    let archetype = self.archetype_index.get_mut(&type_).unwrap();

    for (i, c) in type_.iter().enumerate() {
      let ci = self.component_index.entry(*c).or_default();
      ci.insert(
        archetype.id,
        ArchetypeRecord {
          column: i,
          archetype: UnsafeArchetypeCell::new(archetype),
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
    let component = row.get(a_record.column)?;

    Some(&**component)
  }

  #[allow(unused)]
  pub fn has_comp(&self, entity: EntityId, comp: ComponentId) -> bool {
    let record = self.entity_index.get(&entity).unwrap();
    let archetype = unsafe { record.archetype.archetype() };
    archetype.type_.contains(&comp)
  }

  pub fn add_comp(&mut self, entity: EntityId, comp: Box<dyn Component>) {
    #[cfg(feature = "debug")]
    trace!("Adding Component {:?} to Entity {}", comp.id(), entity);

    let record = self.entity_index.get_mut(&entity).unwrap();
    let from = unsafe { record.archetype.archetype_mut() };

    if from.type_.contains(&comp.id()) {
      return;
    }

    let to = if let Some(to) = from.edges.get(&comp.id()) {
      unsafe { to.add.archetype_mut() }
    } else {
      let mut type_ = from.type_.clone();
      type_.push(comp.id());
      type_.sort_unstable();

      let to = if let Some(to) = self.archetype_index.get_mut(&type_) {
        to
      } else {
        self.create_archetype(type_.clone());
        self.archetype_index.get_mut(&type_).unwrap()
      };

      from.edges.insert(
        comp.id(),
        ArchetypeEdge {
          add: UnsafeArchetypeCell::new(to),
          remove: UnsafeArchetypeCell::null(),
        },
      );

      to
    };

    let record = self.entity_index.get_mut(&entity).unwrap();
    let new_comp = to.type_.iter().position(|&c| c == comp.id()).unwrap();

    to.entity_ids.push(from.entity_ids.swap_remove(record.row));

    let mut entity = from.rows.swap_remove(record.row);
    entity.insert(new_comp, comp);
    to.rows.push(entity);

    let old_row = record.row;
    record.row = to.rows.len() - 1;
    record.archetype = UnsafeArchetypeCell::new(to);

    if let Some(swapped) = from.entity_ids.get(old_row) {
      let swapped_record = self.entity_index.get_mut(swapped).unwrap();
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

    to.entity_ids.push(from.entity_ids.swap_remove(record.row));

    let mut entity = from.rows.swap_remove(record.row);
    entity.remove(removed_comp);
    to.rows.push(entity);

    let old_row = record.row;
    record.row = to.rows.len() - 1;
    record.archetype = UnsafeArchetypeCell::new(to);

    if let Some(swapped) = from.entity_ids.get(old_row) {
      let swapped_record = self.entity_index.get_mut(swapped).unwrap();
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

      if comps.iter().all(|c| archetype.type_.contains(c)) && !archetype.entity_ids.is_empty() {
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

        let comps = archetype.rows.iter_mut().collect();

        result.push(QueryResult {
          columns,
          comps,
          ids: archetype.entity_ids.clone(),
        });
      }
    }

    result
  }

  pub fn get_all_entities_for_archetypes(
    &mut self,
    components: Vec<ComponentId>,
  ) -> VecDeque<(EntityId, &mut Vec<Box<dyn Component>>)> {
    assert!(!components.is_empty());
    let mut entities = VecDeque::new();
    for archetype in &mut self.archetype_index.values_mut() {
      if components.iter().all(|t| archetype.type_.contains(t)) {
        for (e, id) in archetype.rows.iter_mut().zip(archetype.entity_ids.iter()) {
          entities.push_back((*id, e));
        }
      }
    }

    entities
  }
}

#[cfg(test)]
mod test {
  use super::Storage;
  use crate::{self as gravitron_ecs, components::Component};
  use gravitron_ecs_macros::Component;

  #[derive(Component)]
  struct A {}

  #[test]
  fn create_entity() {
    let mut storage = Storage::default();

    storage.create_entity(Vec::new());
  }

  #[test]
  fn remove_entity() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new());
    storage.remove_entity(id);
  }

  #[test]
  fn add_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new());
    storage.add_comp(id, Box::new(A {}));

    assert!(storage.has_comp(id, A::sid()));
  }

  #[test]
  fn remove_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new());
    storage.add_comp(id, Box::new(A {}));
    storage.remove_comp(id, A::sid());

    assert!(!storage.has_comp(id, A::sid()));
  }

  #[test]
  fn has_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new());
    storage.add_comp(id, Box::new(A {}));

    assert!(storage.has_comp(id, A::sid()));
  }

  #[test]
  fn get_comp() {
    let mut storage = Storage::default();

    let id = storage.create_entity(Vec::new());
    storage.add_comp(id, Box::new(A {}));

    let comp = storage.get_comp(id, A::sid()).unwrap();
    let _ = comp.downcast_ref::<A>().unwrap();
  }
}
