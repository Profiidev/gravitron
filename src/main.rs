use gravitron::{
  config::EngineConfig,
  ecs::{systems::{query::Query, resources::Res}, Component},
  ecs_resources::{components::{
    camera::CameraBuilder, renderer::MeshRenderer, transform::Transform,
  }, resources::engine_info::EngineInfo},
  engine::Gravitron,
  math,
  vulkan::graphics::resources::material::Material,
};

fn main() {
  let config = EngineConfig::default();
  let mut builder = Gravitron::builder(config).add_system(test);
  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(10.0, 0.0, 0.0));
  builder.create_entity((
    MeshRenderer {
      model_id: 0,
      material: Material {
        color: math::Vec3::new(1.0, 1.0, 0.0),
        ..Default::default()
      },
    },
    transform,
    Marker::default(),
  ));
  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(0.0, 0.0, 0.0));
  builder.create_entity((
    MeshRenderer {
      model_id: 0,
      material: Material {
        color: math::Vec3::new(0.0, 1.0, 1.0),
        ..Default::default()
      },
    },
    transform,
  ));

  let mut camera_transform = Transform::default();
  camera_transform.set_rotation(
    -std::f32::consts::FRAC_PI_4,
    std::f32::consts::FRAC_PI_4 * 3.0,
    0.0,
  );
  dbg!(camera_transform.rotation() * math::Vec3::X);
  camera_transform.set_position(math::Vec3::new(10.0, 10.0, 10.0));
  builder.create_entity((
    CameraBuilder::new().build(&camera_transform),
    camera_transform,
  ));

  let engine = builder.build();
  engine.run();
}

#[derive(Component, Default)]
pub struct Marker {
  t: f32,
}

fn test(info: Res<EngineInfo>, q: Query<(&mut Transform, &mut Marker)>) {
  for (t, m) in q {
    let mut pos = t.position();
    pos.x = m.t.cos() * 5.0;
    pos.z = m.t.sin() * 5.0;
    t.set_position(pos);
    m.t += 0.5 * info.delta_time();
  }
}
