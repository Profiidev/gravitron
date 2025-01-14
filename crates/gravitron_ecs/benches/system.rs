use std::{hint::black_box, time::Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use gravitron_ecs::{
  systems::{query::Query, IntoSystem, System},
  Component, ECS,
};

fn system_loop(query: Query<&A>) {
  for (_, a) in query {
    black_box(a);
  }
}

fn query_loop_benchmark(c: &mut Criterion) {
  for i in [1, 1000, 1_000_000] {
    let mut ecs = ECS::builder();
    ecs.add_system(system_loop);

    for _ in 0..i {
      ecs.create_entity(A { _x: 0.0 });
    }

    let mut ecs = ecs.build();
    let world = ecs.get_world_cell();
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
