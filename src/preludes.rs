// Internal prelude for use within the crate
pub mod internal {
    pub use crate::components::*;
    pub use crate::resources::*;
    pub use crate::materials::*;
    pub use crate::effects::*;
    pub use crate::systems::*;
    pub use crate::input::*;
    pub use crate::spawners::*;
    pub use crate::hooks::*;

    pub use bevy::{
        color::ColorToComponents,
        ecs::lifecycle::HookContext,
        ecs::world::DeferredWorld,
        log::*,
        mesh::{MeshTag, RectangleMeshBuilder},
        platform::collections::HashMap,
        prelude::*,
        render::{
            render_resource::{AsBindGroup, ShaderType},
            storage::ShaderStorageBuffer,
        },
        sprite_render::{Material2d, Material2dPlugin},
    };
    pub use derive_more::From;
    pub use enum_map::{Enum, EnumMap};
    pub use rand::prelude::*;
    pub use std::collections::{HashSet, VecDeque};
    pub use std::f32::{self};

    // Constants should definitely live here
    pub const MAX_FX: usize = 6;
    pub const MAX_SPATIAL_FX: usize = 3;
    pub const MAX_COLOR_FX: usize = 3;
    pub const MAX_VFX_ENTITIES: usize = 500;
}

// User-facing prelude
pub mod user {
    pub use crate::VfxPlugin;
    pub use crate::components::{Vfx, VfxBundle, VfxBroadcast};
    pub use crate::materials::*;
    pub use crate::effects::*;
    pub use crate::resources::{AtlasDimensions, VfxBroadcastMaterialHandle, VfxMeshHandle};
    pub use crate::systems::{setup_broadcast_material, update_broadcast_effect_stack, spawn_camera, control_2d_camera};
    pub use crate::input::{control_broadcast_fx, play_fx};
    pub use crate::spawners::*;
}
