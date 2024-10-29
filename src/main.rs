use gravitron::{
  config::{
    vulkan::{
      DescriptorSet, DescriptorType, Filter, GraphicsPipelineConfig, ImageConfig, ShaderStageFlags,
      VulkanConfig,
    },
    EngineConfig,
  },
  ecs::{
    commands::Commands,
    components::{
      camera::CameraBuilder,
      lighting::{DirectionalLight, PointLight, SpotLight},
      renderer::MeshRenderer,
      transform::Transform,
    },
    resources::{engine_info::EngineInfo, input::Input},
    systems::{query::Query, resources::Res},
    Component,
  },
  engine::Gravitron,
  math,
  vulkan::graphics::resources::material::Material,
};
use winit::keyboard::KeyCode;

fn main() {
  let testing = GraphicsPipelineConfig::new("testing".to_string())
    .set_frag_shader(vk_shader_macros::include_glsl!("./testing/shader.frag").to_vec())
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
  let config = EngineConfig::default().set_vulkan_config(vulkan);
  let mut builder = Gravitron::builder(config)
    .add_system(test)
    .add_system(test2)
    .add_system(test3);
  let mut transform = Transform::default();
  transform.set_position(math::Vec3::new(5.0, 0.0, 0.0));
  builder.create_entity((
    MeshRenderer {
      model_id: 0,
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
  builder.create_entity((
    MeshRenderer {
      model_id: 0,
      material: Material {
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

  let mut dl_t = Transform::default();
  dl_t.set_rotation(0.0, std::f32::consts::FRAC_PI_4 * 3.0, 0.0);
  builder.create_entity((
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
  builder.create_entity((
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
  builder.create_entity((
    SpotLight {
      color: glam::Vec3::new(0.0, 1.0, 0.0),
      intensity: 1.0,
      range: 1.0,
      angle: 1.0,
    },
    t,
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
      texture_id: 1,
      ..Default::default()
    },
  };
  cmd.create_entity((Transform::default(), Marker::default(), renderer));
}

fn test2(info: Res<EngineInfo>, q: Query<(&mut Transform, &DirectionalLight, &mut Marker)>) {
  for (t, _, m) in q {
    let rot = m.t;
    t.set_rotation(rot, 0.0, rot);
    m.t += 0.05 * info.delta_time();
  }
}

fn test3(input: Res<Input>) {
  if input.is_key_pressed(&KeyCode::KeyW) {
    println!("W");
  }
}
