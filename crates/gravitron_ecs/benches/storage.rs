#[path = "../src/storage.rs"]
#[allow(unused)]
mod storage;

#[path = "../src/tick.rs"]
#[allow(unused)]
mod tick;

pub mod components {
  pub use gravitron_ecs::components::Component;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Default)]
pub struct Id(pub(crate) u64);

impl Display for Id {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
pub type ComponentId = TypeId;
pub type EntityId = Id;
type ArchetypeId = Id;

use std::{any::TypeId, fmt::Display, hint::black_box, time::Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use gravitron_ecs::{components::Component, Component};
use storage::{ComponentBox, Storage};
use tick::Tick;

fn create_n(storage: &mut Storage, n: u64) {
  for _ in 0..n {
    storage.create_entity(vec![Box::new(A { _x: 0.0 })], Tick::default());
  }
}

fn edit_n(storage: &mut Storage, n: u64) {
  for i in 0..n {
    storage.add_comp(
      Id(i),
      ComponentBox {
        comp: Box::new(A { _x: 0.0 }),
        added: Tick::default(),
        changed: Tick::default(),
      },
    );
  }
}

fn get_n(storage: &mut Storage, n: u64) {
  for i in 0..n {
    storage.get_comp(Id(i), TypeId::of::<A>());
  }
}

fn remove_n(storage: &mut Storage, n: u64) {
  for i in 0..n {
    storage.remove_entity(Id(i));
  }
}

fn create_benchmark(c: &mut Criterion) {
  for i in [1, 1000] {
    c.bench_function(&format!("create {}", i), |b| {
      b.iter_custom(|iters| {
        let mut storage = Storage::default();
        let start = Instant::now();
        for _ in 0..iters {
          create_n(&mut storage, black_box(i))
        }
        start.elapsed()
      })
    });
  }
}

fn add_benchmark(c: &mut Criterion) {
  for i in [1, 1000] {
    c.bench_function(&format!("add {}", i), |b| {
      b.iter_custom(|iters| {
        let mut storage = Storage::default();
        create_n(&mut storage, i);

        let start = Instant::now();
        for _ in 0..iters {
          edit_n(&mut storage, black_box(i))
        }
        start.elapsed()
      })
    });
  }
}

fn get_benchmark(c: &mut Criterion) {
  for i in [1, 1000] {
    c.bench_function(&format!("get {}", i), |b| {
      b.iter_custom(|iters| {
        let mut storage = Storage::default();
        create_n(&mut storage, i);

        let start = Instant::now();
        for _ in 0..iters {
          get_n(&mut storage, black_box(i))
        }
        start.elapsed()
      })
    });
  }
}

fn remove_benchmark(c: &mut Criterion) {
  for i in [1, 1000] {
    c.bench_function(&format!("remove {}", i), |b| {
      b.iter_custom(|iters| {
        let mut storage = Storage::default();
        create_n(&mut storage, i);

        let start = Instant::now();
        for _ in 0..iters {
          remove_n(&mut storage, black_box(i))
        }
        start.elapsed()
      })
    });
  }
}

fn query_benchmark(c: &mut Criterion) {
  for i in [1, 1000, 1_000_000] {
    let mut storage = Storage::default();
    create_n(&mut storage, i);

    c.bench_function(&format!("query {}", i), |b| {
      b.iter_custom(|iters| {
        let start = Instant::now();
        for _ in 0..iters {
          let _ = storage.query_data(&[A::sid()], |_| true);
        }
        start.elapsed()
      })
    });
  }
}

criterion_group!(create, create_benchmark);
criterion_group!(add, add_benchmark);
criterion_group!(get, get_benchmark);
criterion_group!(remove, remove_benchmark);
criterion_group!(query, query_benchmark);
criterion_main!(create, add, get, remove, query);

#[derive(Component)]
struct A {
  _x: f32,
}

#[derive(Component)]
struct B {
  _y: f32,
}
