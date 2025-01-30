use gravitron::{
  ecs::{
    commands::Commands,
    components::{
      camera::CameraBuilder,
      lighting::{DirectionalLight, PointLight, SpotLight},
      renderer::MeshRenderer,
      transform::Transform,
    },
    hierarchy::command_ext::HierarchyCommandExt,
    resources::{engine_commands::EngineCommands, engine_info::EngineInfo, input::Input},
    systems::{
      query::{filter::With, Query},
      resources::{Res, ResMut},
    },
    Component,
  },
  engine::Gravitron,
  math,
  plugin::{
    config::vulkan::{
      DescriptorSet, DescriptorType, Filter, GraphicsPipelineConfig, ImageConfig, ShaderStageFlags,
      VulkanConfig,
    },
    Plugin,
  },
  renderer::{
    graphics::resources::{material::Material, model::CUBE_MODEL},
    include_glsl,
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
  fn build(&self, builder: &mut gravitron_plugin::app::AppBuilder<gravitron_plugin::app::Build>) {
    let testing = GraphicsPipelineConfig::new("testing".to_string())
      .set_frag_shader(include_glsl!("./testing/shader.frag").to_vec())
      .add_descriptor_set(
        DescriptorSet::default().add_descriptor(DescriptorType::new_image(
          ShaderStageFlags::FRAGMENT,
          vec![ImageConfig::new_path(
            "./testing/image.png",
            Filter::NEAREST,
          )],
        )),
      );
    let vulkan = VulkanConfig::default()
      .add_graphics_pipeline(testing)
      .add_texture(ImageConfig::new_path(
        "./testing/image.png",
        Filter::NEAREST,
      ));

    builder.config_mut().vulkan = vulkan;

    builder.add_init_system(init);

    builder.add_main_system(test);
    builder.add_main_system(test2);
    builder.add_main_system(test3);
    builder.add_main_system(test4);

    builder.add_resource(Id::default());
    builder.add_resource(false);
  }
}

fn init(cmds: &mut Commands, mut id: ResMut<Id>) {
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
        shader: "testing".into(),
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
  cmds.create_entity((
    CameraBuilder::new().build(&camera_transform),
    camera_transform,
  ));

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
      texture_id: 1,
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
