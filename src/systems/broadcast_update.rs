use crate::internal_prelude::*;

/// System to update the broadcast effect stack
pub fn update_broadcast_effect_stack(
    broadcast_mat_handle: Res<VfxBroadcastMaterialHandle>,
    mut materials: ResMut<Assets<VfxBroadcastMaterial>>,
    time: Res<Time>,
    // You can add your own logic here to determine what effects to broadcast
    // For example, query for a controller entity or resource
) {
    if let Some(material) = materials.get_mut(&broadcast_mat_handle.0) {
        // Example: You could update the effect stack here based on game state
        // material.effect_stack = new_effect_stack;

        // Or prune expired effects
        material.effect_stack.expire(time.elapsed_secs());
    }
}
