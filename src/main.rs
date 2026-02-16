use bevy::prelude::*;
use bevy_hirundo::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            HirundoPlugin::default().with_camera(), // Auto-registers camera spawn & controls
        ))
        .add_systems(Startup, spawn_broadcast_entities)
        .add_systems(Update, control_broadcast_fx)
        .run();
}
