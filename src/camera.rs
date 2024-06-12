use glam as g;
use ash::vk;

use crate::buffer::Buffer;

pub struct CameraBuilder {
  pub position: g::Vec3,
  pub view_direction: g::Vec3,
  pub up: g::Vec3,
  pub fov: f32,
  pub aspect_ratio: f32,
  pub near: f32,
  pub far: f32,
}

impl CameraBuilder {
  pub fn position(mut self, position: g::Vec3) -> Self {
    self.position = position;
    self
  }

  pub fn view_direction(mut self, view_direction: g::Vec3) -> Self {
    self.view_direction = view_direction.normalize();
    self
  }

  pub fn up(mut self, up: g::Vec3) -> Self {
    self.up = up.normalize();
    self
  }

  pub fn fov(mut self, fov: f32) -> Self {
    self.fov = fov.max(0.01).min(std::f32::consts::PI - 0.01);
    self
  }

  pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
    self.aspect_ratio = aspect_ratio;
    self
  }

  pub fn near(mut self, near: f32) -> Self {
    self.near = near;
    self
  }

  pub fn far(mut self, far: f32) -> Self {
    self.far = far;
    self
  }

  pub fn build(self) -> Camera {
    let mut cam = Camera {
      view_matrix: g::Mat4::IDENTITY,
      position: self.position,
      view_direction: self.view_direction,
      up: (self.up - self.view_direction.dot(self.view_direction * self.view_direction)).normalize(),
      fov: self.fov,
      aspect_ratio: self.aspect_ratio,
      near: self.near,
      far: self.far,
      projection_matrix: g::Mat4::IDENTITY,
    };
    cam.update_projection_matrix();
    cam.update_view_matrix();
    cam
  }
}

pub struct Camera {
  pub view_matrix: g::Mat4,
  pub position: g::Vec3,
  pub view_direction: g::Vec3,
  pub up: g::Vec3,
  pub fov: f32,
  pub aspect_ratio: f32,
  pub near: f32,
  pub far: f32,
  pub projection_matrix: g::Mat4,
}

impl Camera {
  pub fn builder() -> CameraBuilder {
    CameraBuilder {
      position: g::Vec3::new(0.0, 3.0, -3.0),
      view_direction: g::Vec3::new(0.0, -1.0, 1.0),
      up: g::Vec3::new(0.0, 1.0, 1.0),
      fov: std::f32::consts::FRAC_PI_3,
      aspect_ratio: 800.0 / 600.0,
      near: 0.1,
      far: 100.0,
    }
  }

  pub fn update_buffer(&self, buffer: &mut Buffer) -> Result<(), vk::Result> {
    let data = [self.view_matrix.to_cols_array_2d(), self.projection_matrix.to_cols_array_2d()];
    buffer.fill(&data)?;
    Ok(())
  }

  fn update_view_matrix(&mut self) {
    self.view_matrix = g::Mat4::look_at_rh(self.position, self.position + self.view_direction, -self.up);
  }

  fn update_projection_matrix(&mut self) {
    self.projection_matrix = g::Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
  }

  pub fn move_forward(&mut self, amount: f32) {
    self.position += self.view_direction * amount;
    self.update_view_matrix();
  }

  pub fn move_backward(&mut self, amount: f32) {
    self.move_forward(-amount);
  }

  pub fn move_right(&mut self, amount: f32) {
    self.position += self.view_direction.cross(-self.up).normalize() * amount;
    self.update_view_matrix();
  }

  pub fn move_left(&mut self, amount: f32) {
    self.move_right(-amount);
  }

  pub fn move_up(&mut self, amount: f32) {
    self.position += self.up * amount;
    self.update_view_matrix();
  }

  pub fn move_down(&mut self, amount: f32) {
    self.move_up(-amount);
  }

  pub fn turn_right(&mut self, amount: f32) {
    let rotation = g::Quat::from_axis_angle(self.up, amount);
    self.view_direction = rotation * self.view_direction;
    self.update_view_matrix();
  }

  pub fn turn_left(&mut self, amount: f32) {
    self.turn_right(-amount);
  }

  pub fn turn_up(&mut self, amount: f32) {
    let rotation = g::Quat::from_axis_angle(self.view_direction.cross(self.up), amount);
    self.view_direction = rotation * self.view_direction;
    self.up = rotation * self.up;
    self.update_view_matrix();
  }

  pub fn turn_down(&mut self, amount: f32) {
    self.turn_up(-amount);
  }
}
