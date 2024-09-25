use std::{
  sync::{atomic::AtomicUsize, Arc},
  thread,
  time::Duration,
};

use graph::Graph;
use gravitron_utils::thread::ThreadPool;

use crate::{
  systems::{IntoSystem, StoredSystem, System},
  world::{UnsafeWorldCell, World},
};

mod graph;

type Stage = Vec<StoredSystem>;

pub struct Scheduler {
  systems: Vec<Stage>,
  thread_pool: ThreadPool,
}

#[derive(Default)]
pub struct SchedulerBuilder {
  systems: Vec<StoredSystem>,
}

impl Scheduler {
  pub fn run(&mut self, world: &mut World) {
    for stage in self.systems.iter_mut() {
      let running = Arc::new(AtomicUsize::new(stage.len()));
      for system in stage {
        let world_cell = UnsafeWorldCell::new(world);
        let running = running.clone();
        let system: &mut Box<dyn System + 'static> = unsafe { std::mem::transmute(system) };
        self.thread_pool.execute(move || {
          system.run(world_cell);
          running.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
      }
      while running.load(std::sync::atomic::Ordering::SeqCst) != 0 {
        thread::sleep(Duration::from_micros(1));
      }
    }
    world.execute_commands();
  }
}

impl SchedulerBuilder {
  pub fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System = S>) {
    self.systems.push(Box::new(system.into_system()));
  }

  pub fn build(self, sync_system_exec: bool) -> Scheduler {
    let stages = if sync_system_exec {
      let mut stages = Vec::new();
      for system in self.systems {
        stages.push(vec![system]);
      }
      stages
    } else {
      let meta_data = self
        .systems
        .iter()
        .map(|s| s.get_meta())
        .collect::<Vec<_>>();
      let graph: Graph = meta_data.into();
      let colored = graph.color();

      let mut stages = (0..colored.num_colors())
        .map(|_| vec![])
        .collect::<Vec<_>>();
      for (i, system) in self.systems.into_iter().enumerate() {
        stages.get_mut(colored.get_color(i)).unwrap().push(system);
      }

      stages
    };

    let longest = stages.iter().map(|s| s.len()).max().unwrap();

    Scheduler {
      systems: stages,
      thread_pool: ThreadPool::new(longest),
    }
  }
}
