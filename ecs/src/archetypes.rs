use crate::Id;

#[derive(Debug, Default)]
pub struct Archetypes {
  types: Vec<Archetype>,
  highest_id: Id,
  unused: Vec<Id>,
}

#[derive(Debug, Default)]
pub struct Archetype {
  id: Id,
  components: Vec<Id>,
}

impl Archetypes {
  pub fn get(&mut self, ids: Vec<Id>) -> Id {
    let found = self.types.iter().find(|a| a.components == ids);

    if let Some(type_) = found {
      type_.id
    } else {
      let id = if let Some(id) = self.unused.pop() {
        id
      } else {
        self.highest_id += 1;
        self.highest_id
      };

      self.types.push(Archetype {
        id,
        components: ids,
      });

      id
    }
  }
}
