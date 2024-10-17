pub struct LightInfo {
  pub num_point_lights: u32,
  pub num_spot_lights: u32,
  pub directional_light: DirectionalLight,
}

#[derive(Default)]
pub struct DirectionalLight {
  pub direction: glam::Vec3,
  pub color: glam::Vec3,
  pub intensity: f32,
}

pub struct PointLight {
  pub position: glam::Vec3,
  pub color: glam::Vec3,
  pub intensity: f32,
  pub range: f32,
}

pub struct SpotLight {
  pub position: glam::Vec3,
  pub direction: glam::Vec3,
  pub color: glam::Vec3,
  pub intensity: f32,
  pub range: f32,
  pub angle: f32,
}
