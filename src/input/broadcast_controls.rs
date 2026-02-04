use crate::internal_prelude::*;

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
