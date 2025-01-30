use gravitron_ecs::Component;
use gravitron_hierarchy::propagation::PropagationUpdate;

#[derive(Component)]
pub struct Transform {
  position: glam::Vec3,
  rotation: glam::Quat,
  scaling: glam::Vec3,
}

#[derive(Component, Clone)]
pub struct GlobalTransform {
  position: glam::Vec3,
  rotation: glam::Quat,
  scaling: glam::Vec3,
  position_matrix: glam::Mat4,
  inverse_position_matrix: glam::Mat4,
}

impl Transform {
  pub fn position(&self) -> glam::Vec3 {
    self.position
  }

  pub fn rotation(&self) -> glam::Quat {
    self.rotation
  }

  pub fn scale(&self) -> glam::Vec3 {
    self.scaling
  }

  pub fn set_position(&mut self, position: glam::Vec3) {
    self.position = position;
  }

  pub fn set_scale(&mut self, scaling: glam::Vec3) {
    self.scaling = scaling;
  }

  pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
    self.rotation = glam::Quat::from_euler(glam::EulerRot::ZXYEx, z, x, y);
  }
}

impl Default for Transform {
  fn default() -> Self {
    Self {
      position: Default::default(),
      rotation: glam::Quat::IDENTITY,
      scaling: glam::Vec3::ONE,
    }
  }
}

impl Default for GlobalTransform {
  fn default() -> Self {
    Self {
      position: Default::default(),
      rotation: glam::Quat::IDENTITY,
      scaling: glam::Vec3::ONE,
      position_matrix: glam::Mat4::IDENTITY,
      inverse_position_matrix: glam::Mat4::IDENTITY.inverse(),
    }
  }
}

impl GlobalTransform {
  pub fn matrix(&self) -> glam::Mat4 {
    self.position_matrix
  }

  pub fn inv_matrix(&self) -> glam::Mat4 {
    self.inverse_position_matrix
  }

  pub fn position(&self) -> glam::Vec3 {
    self.position
  }

  pub fn rotation(&self) -> glam::Quat {
    self.rotation
  }

  pub fn scale(&self) -> glam::Vec3 {
    self.scaling
  }

  fn update_position_matrix(&mut self) {
    self.position_matrix =
      glam::Mat4::from_scale_rotation_translation(self.scaling, self.rotation, self.position);
    self.inverse_position_matrix = self.position_matrix.inverse();
  }
}

impl PropagationUpdate for GlobalTransform {
  type Data = Transform;

  fn copy(&self) -> Self {
    self.clone()
  }

  fn update(&mut self, data: &Self::Data) {
    self.position += data.position;
    self.rotation *= data.rotation;
    self.scaling *= data.scaling;
  
    self.update_position_matrix();
  }
}
