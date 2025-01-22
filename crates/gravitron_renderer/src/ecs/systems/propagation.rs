use gravitron_ecs::commands::Commands;
use gravitron_hierarchy::propagation::UpdatePropagationQuery;

use crate::ecs::components::transform::{GlobalTransform, Transform};

pub fn transform_propagate(
  query: UpdatePropagationQuery<Transform, GlobalTransform>,
  cmds: &mut Commands,
) {
  query.propagate(cmds);
}
