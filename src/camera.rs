use crate::internal_prelude::*;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Pan (WASD) and zoom (Z/X)
pub fn control_2d_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Projection, &mut Transform), With<Camera2d>>,
) {
    let Ok((mut projection, mut transform)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    if let Projection::Orthographic(ref mut ortho) = *projection {
        // Zoom (O = out, P = in)
        if keys.pressed(KeyCode::KeyX) {
            ortho.scale *= 4.0f32.powf(dt);
        }
        if keys.pressed(KeyCode::KeyZ) {
            ortho.scale *= 0.25f32.powf(dt);
        }

        // Pan (WASD), normalized so diagonals aren't faster
        let mut dir = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }

        if dir != Vec2::ZERO {
            dir = dir.normalize();

            // World units per second at ortho.scale == 1.0
            // Multiply by ortho.scale to keep screen-space pan feel roughly constant as you zoom.
            let base_speed = 900.0;
            let speed = base_speed * ortho.scale;

            transform.translation += (dir * speed * dt).extend(0.0);
        }
    }
}
