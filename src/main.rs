use gravitron::{
  config::{
    vulkan::{
      DescriptorSet, DescriptorType, GraphicsPipelineConfig, ShaderStageFlags, VulkanConfig,
    },
    EngineConfig,
  },
  ecs::{
    systems::{query::Query, resources::Res},
    Component,
  },
  ecs_resources::{
    components::{camera::CameraBuilder, renderer::MeshRenderer, transform::Transform},
    resources::engine_info::EngineInfo,
  },
  engine::Gravitron,
  math,
  vulkan::graphics::resources::material::Material,
};
use gravitron_ecs::commands::Commands;

fn main() {
  let testing = GraphicsPipelineConfig::new("testing".to_string())
    .set_frag_shader(vk_shader_macros::include_glsl!("./shaders/shader copy.frag").to_vec())
    .add_descriptor_set(
      DescriptorSet::default()
        .add_descriptor(DescriptorType::new_storage(ShaderStageFlags::FRAGMENT, 144))
        .add_descriptor(DescriptorType::new_image(
          ShaderStageFlags::FRAGMENT,
          "./assets/image.png",
        )),
    );
  let vulkan = VulkanConfig::default().add_graphics_pipeline(testing);
  let config = EngineConfig::default().set_vulkan_config(vulkan);
  let mut builder = Gravitron::builder(config).add_system(test);
  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(5.0, 0.0, 0.0));
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
        shader: "testing".into(),
        ..Default::default()
      },
    },
    transform,
  ));

  let mut camera_transform = Transform::default();
  camera_transform.set_rotation(
    0.0,
    std::f32::consts::FRAC_PI_4 * 3.0,
    -std::f32::consts::FRAC_PI_4,
  );
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

fn test(cmd: &mut Commands, info: Res<EngineInfo>, q: Query<(&mut Transform, &mut Marker)>) {
  for (t, m) in q {
    let mut pos = t.position();
    pos.x = m.t.cos() * 5.0;
    pos.z = m.t.sin() * 5.0;
    t.set_position(pos);
    m.t += 0.5 * info.delta_time();
  }
  let renderer = MeshRenderer {
    model_id: 0,
    material: Material {
      color: glam::Vec3::new(0.5, 0.5, 0.5),
      ..Default::default()
    },
  };
  cmd.create_entity((Transform::default(), Marker::default(), renderer));
}
