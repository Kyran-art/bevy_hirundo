use bevy::prelude::*;
use my_bevy_game::{
    internal_prelude::*,
    prelude::*,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        VfxPlugin::default(),
        Material2dPlugin::<VfxBroadcastMaterial>::default(),
    ));
    app.add_systems(PreStartup, setup_broadcast_material);
    app.add_systems(Startup, (spawn_broadcast_entities, spawn_camera));
    app.add_systems(
        Update,
        (
            control_2d_camera,
            control_broadcast_fx,
            update_broadcast_effect_stack,
        ),
    );
    app.run();
}
