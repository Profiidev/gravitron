use std::time::Instant;

use gravitron_ecs::{
  systems::{query::Query, IntoSystem, System},
  world::{UnsafeWorldCell, World},
  Component,
};

#[derive(Component)]
struct A {}

#[derive(Component)]
struct B {}

fn main() {
  let mut world = World::default();

  for _ in 0..1000000 {
    world.create_entity((A {}, B {}));
  }

  let start = Instant::now();

  let mut system = system.into_system();

  for _ in 0..100 {
    system.run(UnsafeWorldCell::new(&mut world));
  }

  println!("{:?}", start.elapsed());
}

fn system(q: Query<(&A, &mut B)>) {
  for _ in q {}
}
