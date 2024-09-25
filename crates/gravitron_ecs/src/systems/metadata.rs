use core::panic;
use std::{
  any::{type_name, TypeId},
  collections::HashMap,
};

use crate::{components::Component, ComponentId};

#[derive(Default)]
pub struct SystemMeta {
  querys: QueryMeta,
  res: HashMap<TypeId, AccessType>,
  cmds: bool,
}

#[derive(Default)]
pub struct QueryMeta {
  comps: HashMap<ComponentId, AccessType>,
  names: HashMap<ComponentId, String>,
  id: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AccessType {
  Write,
  Read,
}

impl SystemMeta {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_res<R: 'static>(&mut self, access: AccessType) {
    match self.res.get(&TypeId::of::<R>()) {
      Some(_) => {
        panic!(
          "System Access Error: Cannot access resource {} multiple times in the same system",
          type_name::<R>()
        );
      }
      None => {
        self.res.insert(TypeId::of::<R>(), access);
      }
    }
  }

  pub fn add_query(&mut self, query: QueryMeta) {
    for (c, a) in query.comps {
      match self.querys.comps.get(&c) {
        Some(&AccessType::Read) => {
          if a == AccessType::Write {
            panic!("Systen Access Error: Cannot access component {} mutable in the same system if it is already used immutable", query.names.get(&c).unwrap());
          }
        }
        Some(&AccessType::Write) => {
          panic!("Systen Access Error: Cannot access component {} multiple times in the same system if it is already used mutable", query.names.get(&c).unwrap());
        }
        None => {
          self.querys.comps.insert(c, a);
          self
            .querys
            .names
            .insert(c, query.names.get(&c).unwrap().clone());
        }
      }
    }
  }

  pub fn add_cmds(&mut self) {
    if self.cmds {
      panic!("Systen Access Error: Cannot access commands multiple times in the same system");
    } else {
      self.cmds = true;
    }
  }

  pub fn overlaps(&self, other: &SystemMeta) -> bool {
    let mut overlap = false;
    for (comp, access) in &self.querys.comps {
      if let Some(other_access) = other.querys.comps.get(comp) {
        overlap = *access == AccessType::Write || *other_access == AccessType::Write || overlap;
      }
    }

    for (id, access) in &self.res {
      if let Some(other_access) = other.res.get(id) {
        overlap = *access == AccessType::Write || *other_access == AccessType::Write || overlap;
      }
    }

    overlap
  }
}

impl QueryMeta {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_comp<C: Component + 'static>(&mut self, access: AccessType) {
    match self.comps.get(&C::sid()) {
      Some(_) => {
        panic!(
          "System Access Error: Cannot access component {} multiple times in the same query",
          type_name::<C>()
        );
      }
      None => {
        self.comps.insert(C::sid(), access);
        self.names.insert(C::sid(), type_name::<C>().to_string());
      }
    }
  }

  pub fn use_id(&mut self) {
    if !self.comps.is_empty() {
      panic!("System Access Error: Can only use EntityId in the first position");
    }
    if self.id {
      panic!("System Access Error: Can only use EntityId once");
    }
    self.id = true;
  }
}

#[cfg(test)]
mod test {
  use gravitron_ecs_macros::Component;
  use crate as gravitron_ecs;

  use super::{AccessType, QueryMeta, SystemMeta};

  #[derive(Component)]
  struct A;

  #[derive(Component)]
  struct B;

  #[test]
  fn meta_res() {
    let mut meta = SystemMeta::new();

    meta.add_res::<i32>(AccessType::Read);
    meta.add_res::<u32>(AccessType::Write);
    meta.add_res::<String>(AccessType::Write);
  }

  #[test]
  #[should_panic]
  fn meta_res_panic_rr() {
    let mut meta = SystemMeta::new();

    meta.add_res::<i32>(AccessType::Read);
    meta.add_res::<i32>(AccessType::Read);
  }

  #[test]
  #[should_panic]
  fn meta_res_panic_rw() {
    let mut meta = SystemMeta::new();

    meta.add_res::<i32>(AccessType::Read);
    meta.add_res::<i32>(AccessType::Write);
  }

  #[test]
  #[should_panic]
  fn meta_res_panic_ww() {
    let mut meta = SystemMeta::new();

    meta.add_res::<i32>(AccessType::Write);
    meta.add_res::<i32>(AccessType::Write);
  }

  #[test]
  fn meta_query() {
    let mut meta = SystemMeta::new();

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Read);
    query.add_comp::<B>(AccessType::Write);
    meta.add_query(query);

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Read);
    meta.add_query(query);
  }

  #[test]
  #[should_panic]
  fn meta_query_panic_wr() {
    let mut meta = SystemMeta::new();

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Write);
    query.add_comp::<B>(AccessType::Read);
    meta.add_query(query);

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Read);
    meta.add_query(query);
  }

  #[test]
  #[should_panic]
  fn meta_query_panic_ww() {
    let mut meta = SystemMeta::new();

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Write);
    query.add_comp::<B>(AccessType::Read);
    meta.add_query(query);

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Write);
    meta.add_query(query);
  }

  #[test]
  fn query_cmds() {
    let mut meta = SystemMeta::new();
    meta.add_cmds();
  }

  #[test]
  #[should_panic]
  fn query_cmds_panic() {
    let mut meta = SystemMeta::new();
    meta.add_cmds();
    meta.add_cmds();
  }

  #[test]
  fn query_overlap() {
    let mut meta = SystemMeta::new();

    let mut query = QueryMeta::new();
    query.add_comp::<A>(AccessType::Write);
    query.add_comp::<B>(AccessType::Read);
    meta.add_query(query);

    meta.add_res::<i32>(AccessType::Read);
    meta.add_res::<u32>(AccessType::Write);

    let mut meta_2 = SystemMeta::new();

    let mut query = QueryMeta::new();
    query.add_comp::<B>(AccessType::Read);
    meta_2.add_query(query);

    meta_2.add_res::<i32>(AccessType::Read);
  
    assert!(!meta.overlaps(&meta_2));

    meta_2.add_res::<u32>(AccessType::Write);

    assert!(meta.overlaps(&meta_2));

    let mut meta_2 = SystemMeta::new();

    let mut query = QueryMeta::new();
    query.add_comp::<B>(AccessType::Read);
    query.add_comp::<A>(AccessType::Write);
    meta_2.add_query(query);

    assert!(meta.overlaps(&meta_2));
  }

  #[test]
  fn query_comp() {
    let mut query = QueryMeta::new();

    query.use_id();
    query.add_comp::<A>(AccessType::Read);
    query.add_comp::<B>(AccessType::Write);
  }

  #[test]
  #[should_panic]
  fn query_comp_rr() {
    let mut query = QueryMeta::new();

    query.add_comp::<A>(AccessType::Read);
    query.add_comp::<A>(AccessType::Read);
  }

  #[test]
  #[should_panic]
  fn query_comp_rw() {
    let mut query = QueryMeta::new();

    query.add_comp::<A>(AccessType::Read);
    query.add_comp::<A>(AccessType::Write);
  }

  #[test]
  #[should_panic]
  fn query_comp_ww() {
    let mut query = QueryMeta::new();

    query.add_comp::<A>(AccessType::Write);
    query.add_comp::<A>(AccessType::Write);
  }

  #[test]
  #[should_panic]
  fn query_id_not_first() {
    let mut query = QueryMeta::new();

    query.add_comp::<A>(AccessType::Write);
    query.use_id();
  }

  #[test]
  #[should_panic]
  fn query_id_dupe() {
    let mut query = QueryMeta::new();

    query.use_id();
    query.use_id();
  }
}
