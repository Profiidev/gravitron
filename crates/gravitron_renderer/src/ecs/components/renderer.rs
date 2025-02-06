use gravitron_ecs::Component;

use crate::{model::model::ModelHandle, renderer::resources::material::Material};

#[derive(Component)]
pub struct MeshRenderer {
  pub model_id: ModelHandle,
  pub material: Material,
}
