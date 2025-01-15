use std::{hint::black_box, time::Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use gravitron_ecs::{
  systems::{query::Query, IntoSystem, System},
  world::{UnsafeWorldCell, World},
  Component,
};

fn system_loop(query: Query<&A>) {
  for (_, a) in query {
    black_box(a);
  }
}

fn query_loop_benchmark(c: &mut Criterion) {
  for i in [1, 1000, 1_000_000] {
    let mut world = World::new();

    for _ in 0..i {
      world.create_entity(A { _x: 0.0 });
    }

    let world = UnsafeWorldCell::new(&mut world);
    let mut system = system_loop.into_system();

    c.bench_function(&format!("query_loop {}", i), |b| {
      b.iter_custom(|iters| {
        let start = Instant::now();
        for _ in 0..iters {
          system.run(world);
        }
        start.elapsed()
      })
    });
  }
}

criterion_group!(query_loop, query_loop_benchmark);
criterion_main!(query_loop);

#[derive(Component)]
struct A {
  _x: f32,
}
