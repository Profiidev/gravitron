use std::collections::BTreeMap;

use super::types::BufferId;

pub struct BufferMemory {
  offset: usize,
  size: usize,
  buffer: BufferId,
}

pub struct Allocator {
  free: BTreeMap<usize, usize>,
  size: usize,
}

impl BufferMemory {
  #[allow(dead_code)]
  pub fn offset(&self) -> usize {
    self.offset
  }

  #[allow(dead_code)]
  pub fn size(&self) -> usize {
    self.size
  }

  #[allow(dead_code)]
  pub fn buffer(&self) -> BufferId {
    self.buffer
  }
}

impl Allocator {
  #[allow(dead_code)]
  pub fn new(size: usize) -> Self {
    let mut free = BTreeMap::new();
    free.insert(0, size);

    Self { free, size }
  }

  pub fn alloc(&mut self, size: usize, buffer: BufferId) -> Option<BufferMemory> {
    assert!(size > 0);
    let offset = self
      .free
      .iter()
      .find_map(|(o, s)| if *s >= size { Some(*o) } else { None })?;
    let block_size = self.free.remove(&offset)?;

    if block_size > size {
      self.free.insert(offset + size, block_size - size);
    }

    Some(BufferMemory {
      offset,
      size,
      buffer,
    })
  }

  pub fn free(&mut self, offset: usize, size: usize) {
    let before = self
      .free
      .range(..offset)
      .next_back()
      .filter(|(&o, &s)| o + s == size);
    let after = self.free.range(offset..offset + size).next();

    match (before, after) {
      (Some((&o_offset, &o_size)), Some((&a_offset, &a_size))) => {
        self.free.remove(&a_offset);
        self.free.insert(o_offset, o_size + size + a_size);
      }
      (None, Some((&a_offset, &a_size))) => {
        self.free.remove(&a_offset);
        self.free.insert(offset, size + a_size);
      }
      (Some((&o_offset, &o_size)), None) => {
        self.free.insert(o_offset, o_size + size);
      }
      (None, None) => {
        self.free.insert(offset, size);
      }
    }
  }

  pub fn grow(&mut self, additional_size: usize) {
    if let Some((offset, size)) = self.free.iter_mut().next_back() {
      if *offset + *size == self.size {
        *size += additional_size;
        self.size += additional_size;
        return;
      }
    }
    self.free.insert(self.size, additional_size);
    self.size += additional_size;
  }
}
