#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default, Debug)]
pub struct Tick(u64);

impl Tick {
  #[inline]
  pub const fn last(&self) -> Tick {
    Tick(self.0 - 1)
  }

  #[inline]
  pub const fn next(&self) -> Tick {
    Tick(self.0 + 1)
  }
}
