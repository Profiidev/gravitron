use crate::ecs::Component;
use crate::vulkan::graphics::resources::material::Material;
use crate::Id;

#[derive(Component)]
pub struct MeshRenderer {
  pub model_id: Id,
  pub material: Material,
}
