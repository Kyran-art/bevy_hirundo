use crate::internal_prelude::*;

/// Key-based testing for effects
pub fn play_fx(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Vfx>,
    mut q_visible: Query<(Entity, &mut Visibility), With<Vfx>>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        let mut rng = rand::rng();
        for mut vfx in &mut query {
            let random_color = LinearRgba::rgb(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
            );
            vfx.push_effect(
                EffectBuilder::looping(time.elapsed_secs(), 1.0)
                    .color(random_color)
                    .with(Wave::sine(1.0, -0.5, 0.5))
                    .with(BlendMode::Add)
                    .build(),
            );
        }
    } else if input.just_pressed(KeyCode::KeyO) {
        let mut rng = rand::rng();
        for mut vfx in &mut query {
            let random_squash = rng.random_range(-0.5..0.0);
            vfx.push_effect(
                EffectBuilder::one_shot(time.elapsed_secs(), 0.5)
                    .scale_y(-1.0)
                    .with(Wave::sine(1.0, -random_squash, random_squash))
                    .with(Anchor::BottomCenter)
                    .scale_x(1.0)
                    .with(Wave::sine(1.0, random_squash, -random_squash))
                    .build(),
            );
        }
    } else if input.just_pressed(KeyCode::KeyI) {
        let mut rng = rand::rng();
        for mut vfx in &mut query {
            vfx.sprite_index = rng.random_range(0..625);
        }
    } else if input.just_pressed(KeyCode::KeyU) {
        let mut rng = rand::rng();
        let rotations: [f32; 3] = [360.0, 720.0, 1080.0];
        for mut vfx in &mut query {
            let random_degrees = *rotations.choose(&mut rng).unwrap();
            vfx.push_effect(
                EffectBuilder::one_shot(time.elapsed_secs(), 2.0)
                    .rotate(random_degrees)
                    .with(Wave::rotate_continuous(1.0, random_degrees))
                    .with(Envelope::frequency(0.2, 0.0, 0.8).with_ease_out(4.0))
                    .build(),
            );
        }
    } else if input.just_pressed(KeyCode::KeyY) {
        let mut rng = rand::rng();
        let offsets: [f32; 3] = [1.0, 3.0, 5.0];
        for mut vfx in &mut query {
            let offset = *offsets.choose(&mut rng).unwrap();
            vfx.push_effect(
                EffectBuilder::one_shot(time.elapsed_secs(), 1.0)
                    .offset_x(offset)
                    .with(Wave::triangle(1.0, offset, 0.0))
                    .with(WavePhase::center())
                    .build(),
            );
        }
    } else if input.just_pressed(KeyCode::KeyC) {
        for mut vfx in &mut query {
            vfx.clear_effects();
        }
    } else if input.just_pressed(KeyCode::KeyB) {
        info!("KeyB - Toggling entity visibility.");
        for (_, mut vis) in &mut q_visible {
            vis.toggle_visible_hidden();
        }
    } else if input.just_pressed(KeyCode::KeyM) {
        info!("KeyM - Despawning all Vfx entities");
        for (entity, _) in &q_visible {
            commands.entity(entity).despawn();
        }
    } else if input.just_pressed(KeyCode::KeyK) {
        info!("KeyK - Spawning 500 new Vfx entities");
        const COUNT: usize = 500;
        const SPACING: f32 = 50.0;

        let cols: usize = (COUNT as f32).sqrt().ceil() as usize;
        let rows: usize = (COUNT + cols - 1) / cols;
        let total_w = (cols as f32 - 1.0) * SPACING;
        let total_h = (rows as f32 - 1.0) * SPACING;
        let start_x = -total_w * 0.5;
        let start_y = -total_h * 0.5;

        let mut rng = rand::rng();
        for i in 0..COUNT {
            let col = i % cols;
            let row = i / cols;
            let x = start_x + (col as f32) * SPACING;
            let y = start_y + (row as f32) * SPACING;
            let sprite_index = rng.random_range(0..625);

            commands.spawn((Transform::from_xyz(x, y, 0.0), Vfx::new(sprite_index)));
        }
    }
}
