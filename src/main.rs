use std::ops::Deref;

use gravitron::{
  components::{
    camera::CameraBuilder,
    lighting::{DirectionalLight, PointLight, SpotLight},
    renderer::MeshRenderer,
    transform::Transform,
  },
  ecs::{
    commands::Commands,
    hierarchy::command_ext::HierarchyCommandExt,
    systems::{
      query::{filter::With, Query},
      resources::{Res, ResMut},
    },
    Component,
  },
  engine::Gravitron,
  math,
  plugin::{
    app::{AppBuilder, Build},
    ComponentPlugin, Plugin, RendererConfig, RendererPlugin,
  },
  resources::{
    engine_commands::EngineCommands,
    engine_info::EngineInfo,
    input::Input,
    memory::{types::Filter, MemoryManager},
    model::model::CUBE_MODEL,
    pipeline::{
      descriptor::{DescriptorInfo, DescriptorType, ShaderStageFlags},
      graphics::GraphicsPipelineBuilder,
      include_glsl, DescriptorManager, PipelineManager,
    },
    renderer::{resources::material::Material, TextureId, DEFAULT_DESCRIPTOR_SET},
  },
  window::winit::keyboard::KeyCode,
  Id,
};

fn main() {
  let mut builder = Gravitron::builder();
  builder.add_plugin(Game);
  let engine = builder.build();
  engine.run();
}

#[derive(Component, Default)]
pub struct Marker {
  t: f32,
}

#[derive(Component)]
pub struct Center;

struct Game;

impl Plugin for Game {
  fn build(&self, builder: &mut AppBuilder<Build>) {
    builder.add_init_system(init);

    builder.add_main_system(test);
    builder.add_main_system(test2);
    builder.add_main_system(test3);
    builder.add_main_system(test4);

    builder.add_resource(Id::default());
    builder.add_resource(false);

    let texture = builder
      .config_mut::<RendererConfig>()
      .unwrap()
      .graphics
      .add_texture(
        include_bytes!("../testing/image.png").to_vec(),
        Filter::NEAREST,
      );
    builder.add_resource(texture);
  }

  fn dependencies(&self) -> Vec<gravitron_plugin::PluginID> {
    vec![RendererPlugin.id(), ComponentPlugin.id()]
  }
}

fn init(
  cmds: &mut Commands,
  mut id: ResMut<Id>,
  mut pipeline_manager: ResMut<PipelineManager>,
  mut descriptor_manager: ResMut<DescriptorManager>,
  mut memory_manager: ResMut<MemoryManager>,
) {
  let image = memory_manager
    .create_texture_image(Filter::NEAREST, include_bytes!("../testing/image.png"))
    .unwrap();

  let set = descriptor_manager
    .create_descriptor_set(
      vec![DescriptorInfo {
        stage: ShaderStageFlags::FRAGMENT,
        r#type: DescriptorType::Sampler(vec![image]),
      }],
      memory_manager.deref(),
    )
    .unwrap()
    .0;

  let testing = GraphicsPipelineBuilder::new()
    .add_descriptor_set(DEFAULT_DESCRIPTOR_SET)
    .add_descriptor_set(set)
    .fragment_shader(include_glsl!("./testing/shader.frag"));
  let testing = pipeline_manager
    .build_graphics_pipeline(testing, descriptor_manager.deref())
    .unwrap();

  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(5.0, 0.0, 0.0));
  cmds.create_entity((
    MeshRenderer {
      model_id: CUBE_MODEL,
      material: Material {
        color: math::Vec4::new(1.0, 1.0, 0.0, 1.0),
        metallic: 1.0,
        roughness: 0.5,
        ..Default::default()
      },
    },
    transform,
    Marker::default(),
  ));
  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(0.0, 0.0, 0.0));
  *id = cmds.create_entity((
    MeshRenderer {
      model_id: CUBE_MODEL,
      material: Material {
        shader: testing,
        ..Default::default()
      },
    },
    transform,
    Center,
  ));

  let mut camera_transform = Transform::default();
  camera_transform.set_rotation(
    0.0,
    std::f32::consts::FRAC_PI_4 * 3.0,
    -std::f32::consts::FRAC_PI_4,
  );
  camera_transform.set_position(math::Vec3::new(10.0, 10.0, 10.0));
  cmds.create_entity((CameraBuilder::new().build(), camera_transform));

  let mut dl_t = Transform::default();
  dl_t.set_rotation(0.0, std::f32::consts::FRAC_PI_4 * 3.0, 0.0);
  cmds.create_entity((
    DirectionalLight {
      color: glam::Vec3::new(1.0, 0.0, 0.0),
      intensity: 1.0,
      ambient_color: glam::Vec3::new(1.0, 1.0, 1.0),
      ambient_intensity: 0.1,
    },
    dl_t,
    Marker::default(),
  ));

  let mut t = Transform::default();
  t.set_position(glam::Vec3::new(0.0, 1.1, 0.0));
  cmds.create_entity((
    PointLight {
      color: glam::Vec3::new(1.0, 0.0, 1.0),
      intensity: 10.0,
      range: 1.0,
    },
    t,
  ));
  let mut t = Transform::default();
  t.set_position(glam::Vec3::new(5.0, 1.1, 0.0));
  t.set_rotation(std::f32::consts::PI, 0.0, 0.0);
  cmds.create_entity((
    SpotLight {
      color: glam::Vec3::new(0.0, 1.0, 0.0),
      intensity: 1.0,
      range: 1.0,
      angle: 1.0,
    },
    t,
  ));
}

fn test(
  cmd: &mut Commands,
  info: Res<EngineInfo>,
  texture: Res<TextureId>,
  q: Query<(&mut Transform, &mut Marker)>,
  id: Res<Id>,
) {
  for (_, mut t, mut m) in q {
    let mut pos = t.position();
    pos.x = m.t.cos() * 5.0;
    pos.z = m.t.sin() * 5.0;
    t.set_position(pos);
    m.t += 0.5 * info.delta_time();
  }
  let renderer = MeshRenderer {
    model_id: CUBE_MODEL,
    material: Material {
      texture_id: *texture,
      ..Default::default()
    },
  };
  cmd.create_child(*id, (Transform::default(), Marker::default(), renderer));
}

fn test2(info: Res<EngineInfo>, q: Query<(&mut Transform, &DirectionalLight, &mut Marker)>) {
  for (_, mut t, _, mut m) in q {
    let rot = m.t;
    t.set_rotation(rot, 0.0, rot);
    m.t += 0.05 * info.delta_time();
  }
}

fn test3(input: Res<Input>, mut cmds: ResMut<EngineCommands>) {
  if input.is_key_pressed(&KeyCode::Escape) {
    cmds.shutdown();
  }
}

fn test4(query: Query<&mut Transform, With<Center>>, res: Res<EngineInfo>, mut b: ResMut<bool>) {
  for (_, mut t) in query {
    let pos = t.position();
    let mut mov = math::Vec3::new(0.0, 0.2, 0.0) * res.delta_time();

    if pos.y > 2.0 {
      *b = true;
    }
    if pos.y < -2.0 {
      *b = false;
    }

    if *b {
      mov = -mov;
    }

    t.set_position(pos + mov);
  }
}
