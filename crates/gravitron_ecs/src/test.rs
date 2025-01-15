use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc,
};

use gravitron_ecs_macros::Component;

use crate::{
  self as gravitron_ecs,
  commands::Commands,
  systems::query::filter::{Added, Changed, Removed},
};
use crate::{
  scheduler::SchedulerBuilder,
  systems::query::{
    filter::{With, Without},
    Query,
  },
  world::World,
};

#[derive(Component)]
struct A(usize);

#[derive(Component)]
struct B(usize);

fn setup() -> World {
  let mut world = World::new();

  for _ in 0..100 {
    world.create_entity((A(1), B(2)));
    world.create_entity(A(0));
    world.create_entity(B(1000));
  }

  world
}

#[test]
fn test_query() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<&A>| {
    for _ in q {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  builder.add_system(move |q: Query<(&A, &B)>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 400);
  assert_eq!(counter_b.load(Ordering::Relaxed), 200);
}

#[test]
fn test_query_filter_with() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<&A, With<B>>| {
    for _ in q {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  builder.add_system(move |q: Query<(&A, &B), With<A>>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 200);
  assert_eq!(counter_b.load(Ordering::Relaxed), 200);
}

#[test]
fn test_query_filter_without() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<&A, Without<B>>| {
    for _ in q {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  builder.add_system(move |q: Query<(&A, &B), Without<A>>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 200);
  assert_eq!(counter_b.load(Ordering::Relaxed), 0);
}

#[test]
fn test_query_filter_with_without() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<&A, (Without<B>, With<A>)>| {
    for _ in q {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  #[allow(clippy::complexity)]
  builder.add_system(move |q: Query<(&A, &B), (Without<B>, With<A>)>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 200);
  assert_eq!(counter_b.load(Ordering::Relaxed), 0);
}

#[test]
fn test_query_filter_added() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<&A, Without<B>>, cmd: &mut Commands| {
    for (id, _) in q {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);
      cmd.add_comp(id, B(0));
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  #[allow(clippy::complexity)]
  builder.add_system(move |q: Query<(&A, &B), Added<B>>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..4 {
    scheduler.run(&mut world);
    world.next_tick();
  }

  // when creating an entity added is also set
  assert_eq!(counter_a.load(Ordering::Relaxed), 100);
  assert_eq!(counter_b.load(Ordering::Relaxed), 200);
}

#[test]
fn test_query_filter_removed() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<&A, With<B>>, cmd: &mut Commands| {
    for (id, _) in q {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);
      cmd.remove_comp::<B>(id);
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  #[allow(clippy::complexity)]
  builder.add_system(move |q: Query<&A, Removed<B>>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..4 {
    scheduler.run(&mut world);
    world.next_tick();
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 100);
  assert_eq!(counter_b.load(Ordering::Relaxed), 100);
}

#[test]
fn test_query_filter_changed() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<(&mut A, &B)>| {
    for (i, (_, mut a, b)) in q.into_iter().enumerate() {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);

      if i % 2 == 0 {
        a.0 += b.0;
      }
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  #[allow(clippy::complexity)]
  builder.add_system(move |q: Query<&A, Changed<A>>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
    world.next_tick();
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 200);
  assert_eq!(counter_b.load(Ordering::Relaxed), 50);
}

#[test]
fn test_query_filter_combined() {
  let mut world = setup();

  let mut builder: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let counter_a = Arc::new(AtomicUsize::new(0));
  let counter_a_clone = counter_a.clone();
  builder.add_system(move |q: Query<(&mut A, &B)>| {
    for (i, (_, mut a, b)) in q.into_iter().enumerate() {
      counter_a_clone.fetch_add(1, Ordering::Relaxed);

      if i % 2 == 0 {
        a.0 += b.0;
      }
    }
  });

  let counter_b = Arc::new(AtomicUsize::new(0));
  let counter_b_clone = counter_b.clone();
  #[allow(clippy::complexity)]
  builder.add_system(move |q: Query<&A, (Changed<A>, With<B>)>| {
    for _ in q {
      counter_b_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let counter_c = Arc::new(AtomicUsize::new(0));
  let counter_c_clone = counter_c.clone();
  builder.add_system(move |q: Query<&mut A, Without<B>>| {
    for (i, (_, mut a)) in q.into_iter().enumerate() {
      counter_c_clone.fetch_add(1, Ordering::Relaxed);

      if i % 2 == 0 {
        a.0 += 1;
      }
    }
  });

  let counter_d = Arc::new(AtomicUsize::new(0));
  let counter_d_clone = counter_d.clone();
  #[allow(clippy::complexity)]
  builder.add_system(move |q: Query<&A, (Changed<A>, Without<B>)>| {
    for _ in q {
      counter_d_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let mut scheduler = builder.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
    world.next_tick();
  }

  assert_eq!(counter_a.load(Ordering::Relaxed), 200);
  assert_eq!(counter_b.load(Ordering::Relaxed), 50);
  assert_eq!(counter_c.load(Ordering::Relaxed), 200);
  assert_eq!(counter_d.load(Ordering::Relaxed), 50);
}
