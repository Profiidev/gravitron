//! All alignment is required to match the shaders alignment

#[repr(C)]
pub struct LightInfo {
  pub num_point_lights: u32,
  pub num_spot_lights: u32,
  pub directional_light: DirectionalLight,
}

#[repr(C, align(16))]
#[derive(Default)]
pub struct DirectionalLight {
  pub direction: Vec3Align16,
  pub color: glam::Vec3,
  pub intensity: f32,
  pub ambient_color: glam::Vec3,
  pub ambient_intensity: f32,
}

#[repr(C, align(16))]
pub struct PointLight {
  pub position: Vec3Align16,
  pub color: glam::Vec3,
  pub intensity: f32,
  pub range: f32,
}

#[repr(C, align(16))]
pub struct SpotLight {
  pub position: Vec3Align16,
  pub direction: Vec3Align16,
  pub color: glam::Vec3,
  pub intensity: f32,
  pub range: f32,
  pub angle: f32,
}

#[repr(align(16))]
#[derive(Default)]
pub struct Vec3Align16(glam::Vec3);

impl From<glam::Vec3> for Vec3Align16 {
  fn from(value: glam::Vec3) -> Self {
    Vec3Align16(value)
  }
}

impl From<Vec3Align16> for glam::Vec3 {
  fn from(value: Vec3Align16) -> Self {
    value.0
  }
}
