use gravitron_ecs::Component;

use crate::model::model::ModelId;

#[derive(Component)]
pub struct MeshRenderer {
  pub model_id: ModelId,
  pub material: Material,
}
