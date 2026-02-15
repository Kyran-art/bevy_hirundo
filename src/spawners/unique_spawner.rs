use crate::internal_prelude::*;

pub fn spawn_unique_entities(mut commands: Commands) {
    const COUNT: usize = 500;
    const SPACING: f32 = 50.0;

    // Grid dims (near-square) calculation
    let cols: usize = (COUNT as f32).sqrt().ceil() as usize;
    let rows: usize = (COUNT + cols - 1) / cols;

    let total_w = (cols as f32 - 1.0) * SPACING;
    let total_h = (rows as f32 - 1.0) * SPACING;
    let start_x = -total_w * 0.5;
    let start_y = -total_h * 0.5;

    let random_sprite_index = rand::rng().random_range(0..625);

    for i in 0..COUNT {
        let col = i % cols;
        let row = i / cols;

        let x = start_x + (col as f32) * SPACING;
        let y = start_y + (row as f32) * SPACING;
        commands.spawn((
            Transform::from_xyz(x, y, 0.0),
            Vfx::with_sprite(random_sprite_index),
        ));
    }
}
