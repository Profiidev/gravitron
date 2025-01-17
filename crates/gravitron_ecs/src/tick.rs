#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub struct Tick(u64);

impl Tick {
  pub(crate) const INVALID: Tick = Tick(u64::MAX);

  #[inline]
  pub(crate) const fn last(&self) -> Tick {
    Tick(self.0 - 1)
  }

  #[inline]
  pub(crate) const fn next(&self) -> Tick {
    Tick(self.0 + 1)
  }
}

impl Default for Tick {
  fn default() -> Self {
    Tick(1)
  }
}
