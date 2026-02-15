use bevy::prelude::*;
use my_bevy_game::{internal_prelude::*, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        VfxPlugin::default(),
        Material2dPlugin::<VfxBroadcastMaterial>::default(),
    ));
    app.add_systems(PreStartup, setup_vfx_assets);
    app.add_systems(Startup, (spawn_vfx_entities, spawn_camera));
    app.add_systems(Update, (control_2d_camera, play_fx));
    app.run();
}
