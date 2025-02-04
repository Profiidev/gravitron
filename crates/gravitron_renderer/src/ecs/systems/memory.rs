use gravitron_ecs::systems::resources::ResMut;

use crate::memory::MemoryManager;

pub fn reset_buffer_reallocated(mut memory_manager: ResMut<MemoryManager>) {
  memory_manager.reset_reallocated();
}
