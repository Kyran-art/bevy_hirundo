use crate::internal_prelude::*;

/// Helper to spawn a broadcast VFX entity
pub fn spawn_broadcast_entity(
    commands: &mut Commands,
    mesh_handle: &Handle<Mesh>,
    material_handle: &Handle<VfxBroadcastMaterial>,
    transform: Transform,
    sprite_index: u32,
) -> Entity {
    commands
        .spawn((
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(material_handle.clone()),
            transform,
            SpriteIndex(sprite_index),
            VfxBroadcast,
            Visibility::default(),
        ))
        .id()
}

pub fn spawn_broadcast_entities(
    mut commands: Commands,
    mesh_handle: Res<VfxMeshHandle>,
    broadcast_mat_handle: Res<VfxBroadcastMaterialHandle>,
) {
    const COUNT: usize = 20_000;
    info!("Spawning {COUNT} broadcast VFX entities...");
    const SPACING: f32 = 50.0;

    // Grid dims (near-square) calculation
    let cols: usize = (COUNT as f32).sqrt().ceil() as usize;
    let rows: usize = (COUNT + cols - 1) / cols;

    let total_w = (cols as f32 - 1.0) * SPACING;
    let total_h = (rows as f32 - 1.0) * SPACING;
    let start_x = -total_w * 0.5;
    let start_y = -total_h * 0.5;

    let _random_sprite_index = rand::rng().random_range(0..625);

    for i in 0..COUNT {
        let col = i % cols;
        let row = i / cols;

        let x = start_x + (col as f32) * SPACING;
        let y = start_y + (row as f32) * SPACING;
        commands.spawn((
            Mesh2d(mesh_handle.0.clone()),
            MeshMaterial2d(broadcast_mat_handle.0.clone()), // Shared material!
            Transform::from_xyz(x, y, 0.0),
            VfxBroadcast,
            Visibility::default(),
        ));
    }
}
