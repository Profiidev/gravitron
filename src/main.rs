use gravitron::{
  config::EngineConfig,
  ecs_resources::components::{
    camera::CameraBuilder, renderer::MeshRenderer, transform::Transform,
  },
  engine::Gravitron,
  math,
  vulkan::graphics::resources::material::Material,
};

fn main() {
  let config = EngineConfig::default();
  let mut builder = Gravitron::builder(config);
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
  ));
  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(5.0, 0.0, 0.0));
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
  camera_transform.set_rotation(math::Quat::from_axis_angle(math::Vec3::Y, 45_f32.to_radians()));
  camera_transform.set_position(math::Vec3::new(0.0, 0.0, 5.0));
  builder.create_entity((
    CameraBuilder::new().build(&camera_transform),
    camera_transform,
  ));
  let engine = builder.build();
  engine.run();
}
