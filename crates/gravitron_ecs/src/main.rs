use std::time::Instant;

use ecs::{components::Component, query::Query, scheduler::Scheduler, storage::Storage, systems::{Res, ResMut}, world::World};
use ecs_macros::Component;

fn f1(t: Res<usize>, mut r: ResMut<f32>) {
  *r += *t as f32;
}

fn f3(q: Query<(&Transform, &mut Transformw)>) {
  let q = q.into_iter();
  for (e, w) in q {
    w.x += e.x;
  }
}

#[derive(Component)]
struct Transform {
  x: f32,
}

#[derive(Component)]
struct Transformw {
  x: f32,
}

fn main() {
  bench();
  let mut storage = Storage::default();

  storage.create_entity(vec![Box::new(Transform {x: 0.0})]);

  storage.add_comp(0, Box::new(Transform {x: 0.0}));

  storage.remove_comp(0, 0);
  storage.remove_comp(0, 0);
}

fn bench() {
  let mut storage = Storage::default();

  let x = 1000;
  println!("Create Entity");
  let start = Instant::now();

  storage.create_entity(vec![Box::new(Transform {x: 0.0})]);
  println!("{:?}", start.elapsed());

  for _ in 0..x {
    storage.create_entity(vec![Box::new(Transform {x: 0.0})]);
  }

  println!("{:?}", start.elapsed());
  println!("{:?}", start.elapsed() / x);

  println!("Add Component");
  let start = Instant::now();

  storage.add_comp(0, Box::new(Transformw {x: 1.0}));
  println!("{:?}", start.elapsed());

  for i in 1..x {
    storage.add_comp(i as u64, Box::new(Transformw {x: 1.0}));
  }

  println!("{:?}", start.elapsed());
  println!("{:?}", start.elapsed() / x);

  println!("Get Component");
  let start = Instant::now();


  storage.get_comp(0, Transformw::sid());
  println!("{:?}", start.elapsed());

  for i in 0..x {
    storage.get_comp(i as u64, Transformw::sid());
  }

  println!("{:?}", start.elapsed());
  println!("{:?}", start.elapsed() / x);

  println!("Remove Component");
  let start = Instant::now();


  storage.remove_comp(0, Transformw::sid());
  println!("{:?}", start.elapsed());

  for i in 1..x {
    storage.remove_comp(i as u64, Transformw::sid());
  }

  println!("{:?}", start.elapsed());
  println!("{:?}", start.elapsed() / x);

  println!("World");
  let mut world = World::new();

  let x = 1000000;
  println!("Create Entity");
  let start = Instant::now();
  for _ in 0..x {
    world.create_entity((Transform { x: 1.0 }, Transformw { x: 2.0 }));
  }
  println!("{:?}", start.elapsed());
  println!("{:?}", start.elapsed() / x);

  world.add_resource(1usize);
  world.add_resource(0f32);

  let mut scheduler = Scheduler::new();
  scheduler.add_system(f1);
  scheduler.add_system(f3);

  println!("Systems");
  let x = 1000;
  let start = Instant::now();
  for _ in 0..x {
    scheduler.run(&mut world);
  }
  
  println!("{:?}", start.elapsed() / x);
  println!("{:?}", start.elapsed());
}
