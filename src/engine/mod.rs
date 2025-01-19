use std::{
  mem,
  time::{Duration, Instant},
};

use anyhow::Error;
use gravitron_ecs::{
  entity::IntoEntity,
  systems::{IntoSystem, System},
  EntityId,
};
use log::{debug, info, trace};
#[cfg(target_os = "linux")]
use winit::platform::wayland::ActiveEventLoopExtWayland;
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalSize, Size},
  event::{ElementState, KeyEvent},
  event_loop::{ActiveEventLoop, EventLoop},
  keyboard::PhysicalKey,
  window::Window,
};

use crate::{
  config::EngineConfig,
  ecs::{
    resources::{
      engine_commands::EngineCommands, engine_info::EngineInfo, input::Input,
      window::Window as WindowCmds,
    },
    systems::{add_main_systems, stages::SystemStage},
    ECSBuilder, ECS,
  },
  vulkan::Vulkan,
};

pub struct Gravitron {
  config: EngineConfig,
  ecs: ECSEnum,
  window: Option<Window>,
  last_frame: Instant,
  frame_time: Duration,
  input: Input,
}

enum ECSEnum {
  Builder(Box<ECSBuilder<SystemStage>>),
  Ready(ECS),
}

impl Gravitron {
  pub fn builder(config: EngineConfig) -> GravitronBuilder {
    GravitronBuilder::new(config)
  }

  fn init(config: EngineConfig, mut ecs: ECSBuilder<SystemStage>) -> Result<(), Error> {
    info!("Building Engine");

    debug!("Adding ECS Systems");
    add_main_systems(&mut ecs);

    debug!("Configuring EventLoop");
    let mut event_loop = EventLoop::builder();
    let event_loop = event_loop.build()?;

    debug!("Starting Event Loop");
    event_loop.run_app(&mut Gravitron {
      ecs: ECSEnum::Builder(Box::new(ecs)),
      window: None,
      last_frame: Instant::now(),
      frame_time: Duration::from_secs(1) / config.app.fps,
      config,
      input: Default::default(),
    })?;

    Ok(())
  }

  fn ecs_mut(&mut self) -> &mut ECS {
    match &mut self.ecs {
      ECSEnum::Ready(ecs) => ecs,
      _ => unreachable!("Wrong ecs usage"),
    }
  }

  fn ecs_builder_mut(&mut self) -> &mut ECSBuilder<SystemStage> {
    match &mut self.ecs {
      ECSEnum::Builder(ecs) => ecs,
      _ => unreachable!("Wrong ecs usage"),
    }
  }

  fn build_schedulers(&mut self) {
    let temp = unsafe { mem::MaybeUninit::zeroed().assume_init() };
    let builder = mem::replace(&mut self.ecs, temp);
    let builder = match builder {
      ECSEnum::Builder(builder) => builder,
      _ => unreachable!("Ecs already build"),
    };

    let temp = mem::replace(
      &mut self.ecs,
      ECSEnum::Ready(ECS {
        world: builder.world,
        main_scheduler: builder.main_scheduler.build(false),
      }),
    );
    mem::forget(temp);
  }

  fn run(&mut self, event_loop: &ActiveEventLoop) {
    let elapsed = self.last_frame.elapsed();
    if elapsed > self.frame_time {
      self.ecs_mut().world.set_resource(EngineInfo {
        delta_time: elapsed.as_secs_f32(),
      });

      self.last_frame = Instant::now();

      let ecs = self.ecs_mut();
      ecs.main_scheduler.run(&mut ecs.world);

      let cmds = ecs.world.get_resource_mut::<EngineCommands>().unwrap();
      if cmds.is_shutdown() {
        event_loop.exit();
      }

      let new_input = self.input.clone();
      let input = self.ecs_mut().world.get_resource_mut::<Input>().unwrap();
      *input = new_input;

      #[cfg(feature = "debug")]
      trace!("Game loop tok {:?}", self.last_frame.elapsed());
    }
  }
}

impl ApplicationHandler for Gravitron {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes = winit::window::WindowAttributes::default()
      .with_title(self.config.vulkan.title.clone())
      .with_inner_size(Size::Logical(LogicalSize::new(
        self.config.vulkan.width as f64,
        self.config.vulkan.height as f64,
      )));

    debug!("Creating Window");
    let window = event_loop.create_window(window_attributes).unwrap();

    debug!("Creating Vulkan Instnace");
    let vulkan = Vulkan::init(
      std::mem::take(&mut self.config.vulkan),
      &self.config.app,
      &window,
      #[cfg(target_os = "linux")]
      event_loop.is_wayland(),
    )
    .expect("Failed to init Vulkan");

    self.ecs_builder_mut().world.add_resource(vulkan);
    self
      .ecs_builder_mut()
      .world
      .add_resource(EngineInfo::default());
    self
      .ecs_builder_mut()
      .world
      .add_resource(EngineCommands::default());
    self.ecs_builder_mut().world.add_resource(Input::default());
    self
      .ecs_builder_mut()
      .world
      .add_resource(WindowCmds::default());

    self.window = Some(window);

    debug!("Building ECS");
    self.build_schedulers();

    info!("Starting Engine");
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        info!("Stopping Engine");
        event_loop.exit();
      }
      winit::event::WindowEvent::RedrawRequested => {
        self.run(event_loop);
      }
      winit::event::WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(code),
            repeat: false,
            state,
            ..
          },
        ..
      } => match state {
        ElementState::Pressed => {
          self.input.press(code);
        }
        ElementState::Released => {
          self.input.release(&code);
        }
      },
      winit::event::WindowEvent::CursorMoved { position, .. } => {
        self.input.set_cursor_pos(position.x, position.y);
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(window) = &self.window {
      window.request_redraw();
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    debug!("Cleaning up Engine");
    let vulkan = self.ecs_mut().world.get_resource_mut::<Vulkan>().unwrap();
    vulkan.destroy();
  }
}

pub struct GravitronBuilder {
  ecs: ECSBuilder<SystemStage>,
  config: EngineConfig,
}

impl GravitronBuilder {
  pub fn new(config: EngineConfig) -> Self {
    #[cfg(feature = "debug")]
    env_logger::init();

    GravitronBuilder {
      ecs: Default::default(),
      config,
    }
  }

  pub fn add_resource<R: 'static>(mut self, res: R) -> Self {
    self.ecs.world.add_resource(res);
    self
  }

  pub fn add_main_system<I, S: System + 'static>(
    mut self,
    system: impl IntoSystem<I, System = S>,
  ) -> Self {
    self.ecs.main_scheduler.add_system(system);
    self
  }

  pub fn create_entity(&mut self, entity: impl IntoEntity) -> EntityId {
    self.ecs.world.create_entity(entity)
  }

  pub fn run(self) -> Result<(), Error> {
    let Self { config, ecs } = self;
    Gravitron::init(config, ecs)
  }
}
