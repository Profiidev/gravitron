use std::time::Instant;

use ecs::{commands::Commands, components::Component, query::Query, scheduler::Scheduler, storage::Storage, systems::{Res, ResMut}, world::World};
use ecs_macros::Component;

fn f1(t: Res<usize>, mut r: ResMut<f32>) {
  *r += *t as f32;
}

fn f2(r: Res<f32>) {
  println!("{}", *r);
}

fn f3(q: Query<(&Transform, &mut Transformw)>) {
  println!("System Start");
  let start = Instant::now();
  let q = q.into_iter();
  println!("{:?}", start.elapsed());
  println!("Components queried");
  let start = Instant::now();
  for(e, w) in q {
    w.x += e.x;
  }
  println!("{:?}", start.elapsed());
  println!("System End");
}

fn f4(q: Query<(&Transform, &Transformw)>, c: &mut Commands) {
  let (_, w) = q.into_iter().next().unwrap();
  println!("{}", w.x);
  c.create_entity(Transform {x: 1.0})
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

  let x = 10;
  let start = Instant::now();
  for _ in 0..x {
    scheduler.run(&mut world);
  }
  
  let per = start.elapsed() / x;
  let total = start.elapsed();

  scheduler.add_system(f2);
  scheduler.add_system(f4);
  scheduler.run(&mut world);

  println!("{:?}", per);
  println!("{:?}", total);
}
