use ash::vk;
use glam as g;

use crate::buffer::Buffer;

pub struct DirectionalLight {
  pub direction: g::Vec3,
  pub illuminance: [f32; 3],
}

pub struct PointLight {
  pub position: g::Vec3,
  pub illuminance: [f32; 3],
}

pub enum Light {
  Directional(DirectionalLight),
  Point(PointLight),
}

impl From<PointLight> for Light {
  fn from(light: PointLight) -> Self {
    Light::Point(light)
  }
}

impl From<DirectionalLight> for Light {
  fn from(light: DirectionalLight) -> Self {
    Light::Directional(light)
  }
}

#[derive(Default)]
pub struct LightManager {
  pub directional_lights: Vec<DirectionalLight>,
  pub point_lights: Vec<PointLight>,
}

impl  LightManager {
  pub fn add_light<T: Into<Light>>(&mut self, light: T) {
    match light.into() {
      Light::Directional(light) => {
        self.directional_lights.push(light);
      }
      Light::Point(light) => {
        self.point_lights.push(light);
      }
    }
  }

  pub fn update_buffer(
    &self,
    buffer: &mut Buffer
  ) -> Result<(), vk::Result> {
    let mut data = Vec::new();
    data.push(self.directional_lights.len() as f32);
    data.push(self.point_lights.len() as f32);
    data.push(0.0);
    data.push(0.0);
    for light in &self.directional_lights {
      data.push(light.direction.x);
      data.push(light.direction.y);
      data.push(light.direction.z);
      data.push(0.0);
      data.push(light.illuminance[0]);
      data.push(light.illuminance[1]);
      data.push(light.illuminance[2]);
      data.push(0.0);
    }
    for light in &self.point_lights {
      data.push(light.position.x);
      data.push(light.position.y);
      data.push(light.position.z);
      data.push(0.0);
      data.push(light.illuminance[0]);
      data.push(light.illuminance[1]);
      data.push(light.illuminance[2]);
      data.push(0.0);
    }
    buffer.fill(&data)
  }
}

