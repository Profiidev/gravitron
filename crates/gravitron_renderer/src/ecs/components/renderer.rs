use gravitron_ecs::Component;

use crate::graphics::resources::material::Material;
use crate::graphics::resources::model::ModelId;

#[derive(Component)]
pub struct MeshRenderer {
  pub model_id: ModelId,
  pub material: Material,
}
