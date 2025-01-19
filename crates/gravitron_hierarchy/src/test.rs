use std::{
  panic, process,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use gravitron_ecs::{
  commands::Commands,
  scheduler::SchedulerBuilder,
  systems::{query::Query, resources::ResMut},
  world::World,
  Component, Id,
};

use crate::{
  command_ext::HierarchyCommandExt,
  components::{Children, Parent},
};

#[derive(Component)]
struct A {}

#[test]
fn create_child() {
  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  let mut world = World::new();

  let _ = world.create_entity(A {});

  scheduler.add_system(move |cmds: &mut Commands, q: Query<&A>| {
    for (id, _) in q {
      cmds.create_child(id, A {});
    }
  });

  let mut scheduler = scheduler.build(true);

  for _ in 0..4 {
    scheduler.run(&mut world);
  }

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();

  let count_a = Arc::new(AtomicUsize::new(0));
  let count_a_clone = count_a.clone();
  scheduler.add_system(move |q: Query<&A>| {
    for _ in q {
      count_a_clone.fetch_add(1, Ordering::Relaxed);
    }
  });

  let count_b = Arc::new(AtomicUsize::new(0));
  let count_b_clone = count_b.clone();
  scheduler.add_system(move |q: Query<&Parent>| {
    for (_, parent) in q {
      count_b_clone.fetch_add(
        unsafe { std::mem::transmute::<Id, u64>(parent.parent()) } as usize,
        Ordering::Relaxed,
      );
    }
  });

  let count_c = Arc::new(AtomicUsize::new(0));
  let count_c_clone = count_c.clone();
  scheduler.add_system(move |q: Query<&Children>| {
    for (_, children) in q {
      count_c_clone.fetch_add(children.children().len(), Ordering::Relaxed);
    }
  });

  let mut scheduler = scheduler.build(false);

  scheduler.run(&mut world);

  assert_eq!(count_a.load(Ordering::Relaxed), 16);
  assert_eq!(count_b.load(Ordering::Relaxed), 35);
  assert_eq!(count_c.load(Ordering::Relaxed), 15);
}

#[test]
fn set_parent() {
  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  let mut world = World::new();

  let parent1 = world.create_entity(A {});
  let parent2 = world.create_entity(A {});
  let child = world.create_entity(A {});
  world.add_resource(0_usize);

  scheduler.add_system(move |cmds: &mut Commands, mut res: ResMut<usize>| {
    if *res == 0 {
      cmds.set_parent(child, parent1);
      *res += 1;
    } else {
      cmds.set_parent(child, parent2);
    }
  });

  let mut scheduler = scheduler.build(true);

  for _ in 0..2 {
    scheduler.run(&mut world);
  }

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();

  scheduler.add_system(move |cmds: &mut Commands| {
    cmds.custom_fn_command(move |storage, tick| {
      assert!(!storage.has_comp::<Children>(parent1));
      assert!(storage.has_comp::<Children>(parent2));
      assert!(storage.has_comp::<Parent>(child));

      let parent = storage.remove_comp::<Parent>(child, tick).unwrap();
      let children = storage.remove_comp::<Children>(parent2, tick).unwrap();

      assert_eq!(parent.parent(), parent2);
      assert_eq!(children.children(), &[child]);
    });
  });

  let mut scheduler = scheduler.build(true);

  scheduler.run(&mut world);
}

#[test]
fn remove_children() {
  let orig_hook = panic::take_hook();
  panic::set_hook(Box::new(move |panic_info| {
    orig_hook(panic_info);
    process::exit(1);
  }));

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  let mut world = World::new();

  let id = world.create_entity(A {});

  scheduler.add_system(move |cmds: &mut Commands, q: Query<&A>| {
    for (id, _) in q {
      cmds.create_child(id, A {});
    }
  });

  let mut scheduler = scheduler.build(true);

  for _ in 0..4 {
    scheduler.run(&mut world);
  }

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  scheduler.add_system(move |cmds: &mut Commands| {
    cmds.remove_children(id);
  });

  let mut scheduler = scheduler.build(true);
  scheduler.run(&mut world);

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  scheduler.add_system(move |q: Query<&A>| {
    assert_eq!(q.into_iter().collect::<Vec<_>>().len(), 1);
  });

  let mut scheduler = scheduler.build(true);
  scheduler.run(&mut world);
}

#[test]
fn remove_with_children() {
  let orig_hook = panic::take_hook();
  panic::set_hook(Box::new(move |panic_info| {
    orig_hook(panic_info);
    process::exit(1);
  }));

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  let mut world = World::new();

  let id = world.create_entity(A {});

  scheduler.add_system(move |cmds: &mut Commands, q: Query<&A>| {
    for (id, _) in q {
      cmds.create_child(id, A {});
    }
  });

  let mut scheduler = scheduler.build(true);

  for _ in 0..4 {
    scheduler.run(&mut world);
  }

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  scheduler.add_system(move |cmds: &mut Commands| {
    cmds.remove_entity_with_children(id);
  });

  let mut scheduler = scheduler.build(true);
  scheduler.run(&mut world);

  let mut scheduler: SchedulerBuilder<usize> = SchedulerBuilder::default();
  scheduler.add_system(move |q: Query<&A>| {
    assert_eq!(q.into_iter().collect::<Vec<_>>().len(), 0);
  });

  let mut scheduler = scheduler.build(true);
  scheduler.run(&mut world);
}
