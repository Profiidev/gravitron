use core::panic;
use std::{any::{type_name, TypeId}, collections::HashMap};

use crate::{components::Component, storage::ComponentId};

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
  id: bool
}

#[derive(PartialEq, Eq)]
pub enum AccessType {
  Write,
  Read
}

impl SystemMeta {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_res<R: 'static>(&mut self, access: AccessType) {
    match self.res.get(&TypeId::of::<R>()) {
      Some(_) => {
        panic!("System Access Error: Cannot access resource {} multiple times in the same system", type_name::<R>());
      },
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
        },
        Some(&AccessType::Write) => {
          panic!("Systen Access Error: Cannot access component {} multiple times in the same system if it is already used mutable", query.names.get(&c).unwrap());
        },
        None => {
          self.querys.comps.insert(c, a);
          self.querys.names.insert(c, query.names.get(&c).unwrap().clone());
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
}

impl QueryMeta {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add_comp<C: Component + 'static>(&mut self, access: AccessType) {
    match self.comps.get(&C::sid()) {
      Some(_) => {
        panic!("System Access Error: Cannot access component {} multiple times in the same query", type_name::<C>());
      },
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
