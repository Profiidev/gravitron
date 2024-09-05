use std::time::Instant;

use ecs::{query::Query, scheduler::Scheduler, systems::{Res, ResMut}, world::World};
use ecs_macros::Component;

fn f1(t: Res<usize>, mut r: ResMut<f32>) {
  *r += *t as f32;
}

fn f2(r: Res<f32>) {
  println!("{}", *r);
}

fn f3(q: Query<(&Transform, &mut Transformw)>) {
  for(e, w) in q {
    w.x += e.x;
  }
}

fn f4(q: Query<(&Transform, &Transformw)>) {
  let (_, w) = q.into_iter().next().unwrap();
  println!("{}", w.x);
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
  let mut world = World::new();
  for _ in 0..100000 {
    world.add_entity((Transform { x: 1.0 }, Transformw { x: 2.0 }));
  }
  world.add_resource(1usize);
  world.add_resource(0f32);

  let mut scheduler = Scheduler::new();
  scheduler.add_system(f1);
  scheduler.add_system(f3);

  let x = 1000;
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
