use crate::internal_prelude::*;

/// System to update the storage buffer when effect stacks or sprite indices change
pub fn update_effect_storage_buffer(
    mut commands: Commands,
    material_handle: Res<VfxMaterialHandle>,
    mut storage_data: ResMut<EffectStorageData>,
    mut query: Query<(&MeshTag, &Vfx), Changed<Vfx>>,
    mut init_query: Query<(Entity, &mut Visibility), With<VfxGhostBuffer>>,
    mut materials: ResMut<Assets<VfxMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // Only process Changed<Vfx> entities that aren't already dirty
    // (to avoid double-processing entities that were just hydrated)
    for (tag, vfx) in &mut query {
        let index = tag.0 as usize;

        // Skip if already dirty from hydrate/dehydrate hooks
        if storage_data.dirty_slots.contains(&index) {
            continue;
        }

        if index < storage_data.effects.len() {
            let mut updated_stack = vfx.effects.clone();
            updated_stack.tile_index = vfx.sprite_index;
            storage_data.effects[index] = updated_stack;
            storage_data.dirty_slots.insert(index);
        }
    }

    for (entity, mut vis) in &mut init_query {
        *vis = Visibility::Visible;
        commands.entity(entity).remove::<VfxGhostBuffer>();
    }

    // Upload if we have any dirty slots
    if !storage_data.dirty_slots.is_empty() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if let Some(buffer) = buffers.get_mut(&material.effect_storage) {
                buffer.set_data(storage_data.effects.clone());
                storage_data.dirty_slots.clear();
            }
        }
    }
}
