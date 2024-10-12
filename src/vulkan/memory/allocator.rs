use std::collections::BTreeMap;

use super::manager::{AdvancedBufferId, SimpleBufferId};

pub struct AdvancedBufferMemory {
  offset: usize,
  size: usize,
  buffer: AdvancedBufferId,
}

pub struct SimpleBufferMemory {
  offset: usize,
  size: usize,
  buffer: SimpleBufferId,
}

pub struct Allocator {
  free: BTreeMap<usize, usize>,
  size: usize,
}

impl AdvancedBufferMemory {
  pub fn offset(&self) -> usize {
    self.offset
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn buffer(&self) -> AdvancedBufferId {
    self.buffer
  }
}

impl SimpleBufferMemory {
  pub fn offset(&self) -> usize {
    self.offset
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn buffer(&self) -> SimpleBufferId {
    self.buffer
  }
}

impl Allocator {
  pub fn new(size: usize) -> Self {
    let mut free = BTreeMap::new();
    free.insert(0, size);

    Self { free, size }
  }

  pub fn alloc_advanced(&mut self, size: usize, buffer: AdvancedBufferId) -> Option<AdvancedBufferMemory> {
    let offset = self.alloc(size)?;
    Some(AdvancedBufferMemory { offset, size, buffer })
  }

  pub fn alloc_simple(&mut self, size: usize, buffer: SimpleBufferId) -> Option<SimpleBufferMemory> {
    let offset = self.alloc(size)?;
    Some(SimpleBufferMemory { offset, size, buffer })
  }

  fn alloc(&mut self, size: usize) -> Option<usize> {
    assert!(size > 0);
    let offset = self
      .free
      .iter()
      .find_map(|(o, s)| if *s >= size { Some(*o) } else { None })?;
    let block_size = self.free.remove(&offset)?;

    if block_size > size {
      self.free.insert(offset + size, block_size - size);
    }

    Some(offset)
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
