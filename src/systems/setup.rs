use crate::HirundoPlugin;
use crate::internal_prelude::*;

pub fn setup_vfx_assets(
    plugin_config: Res<HirundoPlugin>,
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

/// Setup system for broadcast material (add to PreStartup)
pub fn setup_broadcast_material(
    plugin_config: Res<HirundoPlugin>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<VfxBroadcastMaterial>>,
    mut commands: Commands,
) {
    let material_handle = materials.add(VfxBroadcastMaterial {
        texture: asset_server.load(&plugin_config.texture_path),
        effect_stack: EffectStack::default(),
        atlas_dimensions: plugin_config.atlas_dimensions.clone(),
    });

    commands.insert_resource(VfxBroadcastMaterialHandle(material_handle));
}
