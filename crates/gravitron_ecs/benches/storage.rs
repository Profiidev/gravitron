#[path = "../src/storage.rs"]
#[allow(unused)]
mod storage;

pub mod components {
  pub use gravitron_ecs::components::Component;
}

pub type Id = u64;
pub type ComponentId = TypeId;
pub type EntityId = Id;
type ArchetypeId = Id;

use std::{any::TypeId, hint::black_box, time::Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use gravitron_ecs::Component;
use storage::Storage;

fn create_n(storage: &mut Storage, n: u64) {
  for _ in 0..n {
    storage.create_entity(vec![Box::new(A { _x: 0.0 })]);
  }
}

fn edit_n(storage: &mut Storage, n: u64) {
  for i in 0..n {
    storage.add_comp(i, Box::new(A { _x: 0.0 }));
  }
}

fn get_n(storage: &mut Storage, n: u64) {
  for i in 0..n {
    storage.get_comp(i, TypeId::of::<A>());
  }
}

fn remove_n(storage: &mut Storage, n: u64) {
  for i in 0..n {
    storage.remove_entity(i);
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

criterion_group!(create, create_benchmark);
criterion_group!(add, add_benchmark);
criterion_group!(get, get_benchmark);
criterion_group!(remove, remove_benchmark);
criterion_main!(create, add, get, remove);

#[derive(Component)]
struct A {
  _x: f32,
}

#[derive(Component)]
struct B {
  _y: f32,
}
