use gravitron_ecs::Component;

#[derive(Component)]
pub struct DirectionalLight {
  pub color: glam::Vec3,
  pub intensity: f32,
}

#[derive(Component)]
pub struct PointLight {
  pub color: glam::Vec3,
  pub intensity: f32,
  pub range: f32,
}

#[derive(Component)]
pub struct SpotLight {
  pub color: glam::Vec3,
  pub intensity: f32,
  pub range: f32,
  pub angle: f32,
}
