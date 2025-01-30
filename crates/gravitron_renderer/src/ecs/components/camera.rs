use core::f32;

use gravitron_components::components::transform::Transform;
use gravitron_ecs::Component;

pub struct CameraBuilder {
  fov: f32,
  aspect_ratio: f32,
  near: f32,
  far: f32,
}

impl CameraBuilder {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn fov(mut self, fov: f32) -> Self {
    self.fov = fov.clamp(0.01, std::f32::consts::PI - 0.01);
    self
  }

  #[inline]
  pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
    self.aspect_ratio = aspect_ratio;
    self
  }

  #[inline]
  pub fn near(mut self, near: f32) -> Self {
    self.near = near.max(self.far);
    self
  }

  #[inline]
  pub fn far(mut self, far: f32) -> Self {
    self.far = far.min(self.near);
    self
  }

  pub fn build(self, transform: &Transform) -> Camera {
    let mut cam = Camera {
      view_matrix: glam::Mat4::IDENTITY,
      fov: self.fov,
      aspect_ratio: self.aspect_ratio,
      far: self.far,
      near: self.near,
      projection_matrix: glam::Mat4::IDENTITY,
    };

    cam.update_projection_matrix();
    cam.update_view_matrix(transform);
    cam
  }
}

impl Default for CameraBuilder {
  fn default() -> Self {
    CameraBuilder {
      fov: f32::consts::FRAC_PI_3,
      aspect_ratio: 800.0 / 600.0,
      near: 0.1,
      far: 100.0,
    }
  }
}

#[derive(Component)]
pub struct Camera {
  view_matrix: glam::Mat4,
  fov: f32,
  aspect_ratio: f32,
  near: f32,
  far: f32,
  projection_matrix: glam::Mat4,
}

impl Camera {
  #[inline]
  pub fn builder() -> CameraBuilder {
    CameraBuilder::new()
  }

  #[inline]
  fn update_view_matrix(&mut self, transform: &Transform) {
    self.view_matrix = glam::Mat4::look_at_rh(
      transform.position(),
      transform.position() + transform.rotation() * glam::Vec3::X,
      transform.rotation() * -glam::Vec3::Y,
    );
  }

  #[inline]
  fn update_projection_matrix(&mut self) {
    self.projection_matrix =
      glam::Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
  }

  #[inline]
  pub fn view_matrix(&self) -> glam::Mat4 {
    self.view_matrix
  }

  #[inline]
  pub fn projection_matrix(&self) -> glam::Mat4 {
    self.projection_matrix
  }
}
