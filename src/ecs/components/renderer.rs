use crate::ecs::Component;
use crate::vulkan::graphics::resources::material::Material;
use crate::vulkan::graphics::resources::model::ModelId;

#[derive(Component)]
pub struct MeshRenderer {
  pub model_id: ModelId,
  pub material: Material,
}
