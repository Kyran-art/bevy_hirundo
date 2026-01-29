use bevy::{log::*, prelude::*};
use my_bevy_game::{
    camera::{control_2d_camera, spawn_camera},
    internal_prelude::*,
    prelude::*,
};

// Import the broadcast material module
// Add this to your lib.rs: pub mod broadcast_material;
use my_bevy_game::broadcast_material::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        VfxPlugin::default(),
        // Add the broadcast material plugin
        Material2dPlugin::<VfxBroadcastMaterial>::default(),
    ));
    
    app.add_systems(PreStartup, setup_broadcast_material);
    app.add_systems(
        Startup,
        (spawn_broadcast_vfx_entities, spawn_camera),
    );
    app.add_systems(
        Update,
        (
            control_2d_camera,
            update_broadcast_effects,
            control_broadcast_fx,
        ),
    );
    
    app.run();
}

/// Spawn 10,000+ entities using the broadcast material
fn spawn_broadcast_vfx_entities(
    mut commands: Commands,
    mesh_handle: Res<VfxMeshHandle>,
    broadcast_mat_handle: Res<VfxBroadcastMaterialHandle>,
) {
    const COUNT: usize = 10000; // 10,000 entities!
    const SPACING: f32 = 50.0;

    let cols: usize = (COUNT as f32).sqrt().ceil() as usize;
    let rows: usize = (COUNT + cols - 1) / cols;

    let total_w = (cols as f32 - 1.0) * SPACING;
    let total_h = (rows as f32 - 1.0) * SPACING;
    let start_x = -total_w * 0.5;
    let start_y = -total_h * 0.5;

    let mut rng = rand::rng();
    let random_sprite = rng.random_range(0..625);

    info!("Spawning {} broadcast VFX entities...", COUNT);

    for i in 0..COUNT {
        let col = i % cols;
        let row = i / cols;

        let x = start_x + (col as f32) * SPACING;
        let y = start_y + (row as f32) * SPACING;

        // All entities share the same material, which contains the shared EffectStack
        commands.spawn((
            Mesh2d(mesh_handle.0.clone()),
            MeshMaterial2d(broadcast_mat_handle.0.clone()),
            Transform::from_xyz(x, y, 0.0),
            VfxBroadcast,
            Visibility::default(),
        ));
    }

    info!("Broadcast VFX entities spawned!");
}

/// Update the shared effect stack in the broadcast material
fn update_broadcast_effects(
    broadcast_mat_handle: Res<VfxBroadcastMaterialHandle>,
    mut materials: ResMut<Assets<VfxBroadcastMaterial>>,
    time: Res<Time>,
) {
    if let Some(material) = materials.get_mut(&broadcast_mat_handle.0) {
        // Prune expired effects
        material.effect_stack.expire(time.elapsed_secs());
    }
}

/// Control the broadcast effects with keyboard input
fn control_broadcast_fx(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    broadcast_mat_handle: Res<VfxBroadcastMaterialHandle>,
    mut materials: ResMut<Assets<VfxBroadcastMaterial>>,
) {
    let Some(material) = materials.get_mut(&broadcast_mat_handle.0) else {
        return;
    };

    if input.just_pressed(KeyCode::KeyP) {
        info!("P - Adding pulsing color effect to ALL entities");
        let random_color = LinearRgba::rgb(
            rand::rng().random_range(0.0..1.0),
            rand::rng().random_range(0.0..1.0),
            rand::rng().random_range(0.0..1.0),
        );
        
        material.effect_stack.push(
            EffectBuilder::looping(time.elapsed_secs(), 1.0)
                .color(random_color)
                .with(Wave::sine(1.0, -0.5, 0.5))
                .with(BlendMode::Add)
                .build(),
        );
    } else if input.just_pressed(KeyCode::KeyO) {
        info!("O - Adding squash effect to ALL entities");
        let random_squash = rand::rng().random_range(-0.5..0.0);
        
        material.effect_stack.push(
            EffectBuilder::one_shot(time.elapsed_secs(), 0.5)
                .scale_y(-1.0)
                .with(Wave::sine(1.0, -random_squash, random_squash))
                .with(Anchor::BottomCenter)
                .scale_x(1.0)
                .with(Wave::sine(1.0, random_squash, -random_squash))
                .build(),
        );
    } else if input.just_pressed(KeyCode::KeyI) {
        info!("I - Changing sprite for ALL entities");
        material.effect_stack.tile_index = rand::rng().random_range(0..625);
    } else if input.just_pressed(KeyCode::KeyU) {
        info!("U - Adding rotation effect to ALL entities");
        let rotations: [f32; 3] = [360.0, 720.0, 1080.0];
        let random_degrees = *rotations.choose(&mut rand::rng()).unwrap();
        
        material.effect_stack.push(
            EffectBuilder::one_shot(time.elapsed_secs(), 2.0)
                .rotate(random_degrees)
                .with(Wave::rotate_continuous(1.0, random_degrees))
                .with(Envelope::frequency(0.2, 0.0, 0.8).with_ease_out(4.0))
                .build(),
        );
    } else if input.just_pressed(KeyCode::KeyY) {
        info!("Y - Adding offset shake to ALL entities");
        let offsets: [f32; 3] = [1.0, 3.0, 5.0];
        let offset = *offsets.choose(&mut rand::rng()).unwrap();
        
        material.effect_stack.push(
            EffectBuilder::one_shot(time.elapsed_secs(), 1.0)
                .offset_x(offset)
                .with(Wave::triangle(1.0, offset, 0.0))
                .with(WavePhase::center())
                .build(),
        );
    } else if input.just_pressed(KeyCode::KeyC) {
        info!("C - Clearing all effects");
        material.effect_stack.clear();
    } else if input.just_pressed(KeyCode::KeyR) {
        info!("R - Rainbow wave effect!");
        material.effect_stack.push(
            EffectBuilder::looping(time.elapsed_secs(), 3.0)
                .hue_shift(360.0)
                .with(Wave::sine(1.0, 0.0, 360.0))
                .build(),
        );
    } else if input.just_pressed(KeyCode::KeyT) {
        info!("T - Fade in/out effect!");
        material.effect_stack.push(
            EffectBuilder::looping(time.elapsed_secs(), 2.0)
                .alpha(0.0)
                .with(Wave::sine(1.0, 0.0, 1.0))
                .build(),
        );
    }
}
