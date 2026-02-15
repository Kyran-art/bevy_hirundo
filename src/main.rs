use bevy::prelude::*;
use bevy_hirundo::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            HirundoPlugin::default()
                .with_camera() // Auto-registers camera spawn & controls
                .with_texture("32roguesTextureV2.png")
                .with_atlas(AtlasDimensions {
                    texture_size: Vec2::new(1024.0, 1024.0),
                    cell_size: Vec2::new(40.0, 40.0),
                    sprite_size: Vec2::new(32.0, 32.0),
                    padding: Vec2::new(4.0, 4.0),
                }),
        ))
        .add_systems(Startup, spawn_unique_entities)
        .add_systems(Update, play_fx)
        .run();
}
