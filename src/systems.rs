use crate::{VfxPlugin, internal_prelude::*};

/// System to sync user-facing Vfx component to internal SpriteIndex component
pub fn sync_vfx_to_internal(mut query: Query<(&Vfx, &mut SpriteIndex), Changed<Vfx>>) {
    for (vfx, mut internal_sprite) in &mut query {
        internal_sprite.0 = vfx.sprite_index;
    }
}

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

/// System to prune expired effects (optional - keeps effect stacks clean)
pub fn prune_expired_effects(time: Res<Time>, mut query: Query<&mut Vfx>) {
    let now = time.elapsed_secs();
    for mut vfx in &mut query {
        vfx.effects.expire(now);
    }
}

pub fn setup_vfx_assets(
    plugin_config: Res<VfxPlugin>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VfxMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut mesh_handle_res: ResMut<VfxMeshHandle>,
    mut mat_handle_res: ResMut<VfxMaterialHandle>,
) {
    // 1. Create Mesh
    let mesh_handle = meshes.add(RectangleMeshBuilder::new(
        plugin_config.atlas_dimensions.sprite_size.x,
        plugin_config.atlas_dimensions.sprite_size.y,
    ));
    mesh_handle_res.0 = mesh_handle;

    // 2. Create Storage Buffer
    let buffer_handle = buffers.add(ShaderStorageBuffer::from(vec![
        EffectStack::default();
        MAX_VFX_ENTITIES
    ]));

    // 3. Create Material
    let material_handle = materials.add(VfxMaterial {
        texture: asset_server.load(&plugin_config.texture_path),
        effect_storage: buffer_handle,
        atlas_dimensions: plugin_config.atlas_dimensions.clone(),
    });
    mat_handle_res.0 = material_handle;
}
