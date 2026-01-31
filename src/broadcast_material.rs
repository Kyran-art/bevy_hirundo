use crate::{VfxPlugin, internal_prelude::*};

/// Material for broadcasting a single shared EffectStack to many entities.
/// Unlike VfxMaterial which uses a storage buffer indexed by mesh tag,
/// this material holds one EffectStack as a uniform that all instances share.
///
/// Use this when you want 10,000+ entities to animate with the same effect,
/// achieving better performance through uniform memory access patterns.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VfxBroadcastMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,

    #[uniform(2)]
    pub effect_stack: EffectStack,

    #[uniform(3)]
    pub atlas_dimensions: AtlasDimensions,
}

impl Material2d for VfxBroadcastMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx_broadcast.wgsl".into()
    }
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx_broadcast.wgsl".into()
    }
    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}

/// Component marker for entities using broadcast material
#[derive(Component)]
pub struct VfxBroadcast;

/// Resource holding the broadcast material handle
#[derive(Resource, Deref, DerefMut)]
pub struct VfxBroadcastMaterialHandle(pub Handle<VfxBroadcastMaterial>);

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

/// Setup system for broadcast material (add to PreStartup)
pub fn setup_broadcast_material(
    plugin_config: Res<VfxPlugin>,
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

/// Helper to spawn a broadcast VFX entity
pub fn spawn_broadcast_vfx(
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
/// Control the broadcast effects with keyboard input
pub fn control_broadcast_fx(
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
        info!("Y - Adding wobble to ALL entities");
        let skews: [f32; 3] = [0.3, 0.6, 1.0];
        let skew = *skews.choose(&mut rand::rng()).unwrap();

        material.effect_stack.push(
            EffectBuilder::one_shot(time.elapsed_secs(), 1.0)
                .skew_x(skew)
                .with(Wave::triangle(3.0, skew, 0.0))
                .with(AmplitudeEnvelope::new(0.1, 0.0, 0.9).with_ease_out(4.0))
                .with(WavePhase::center())
                .with(Anchor::BottomCenter)
                .build(),
        );
    } else if input.just_pressed(KeyCode::KeyC) {
        info!("C - Clearing all effects");
        material.effect_stack.clear();
    } else if input.just_pressed(KeyCode::KeyR) {
        info!("R - Blue wave effect!");
        material.effect_stack.push(
            EffectBuilder::looping(time.elapsed_secs(), 3.0)
                .color(LinearRgba::BLUE)
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
