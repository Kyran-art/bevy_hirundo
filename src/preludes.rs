// make private via (crate)
pub mod internal {
    pub use crate::camera::*;
    pub use crate::render::*;
    pub use crate::systems::*;
    pub use crate::vfx::*;
    pub use bevy::{
        color::ColorToComponents,
        log::*,
        mesh::{MeshTag, RectangleMeshBuilder},
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
    pub use std::f32::{self}; // Added HashSet for dirty tracking

    // Constants should definitely live here or in vfx.rs
    pub const MAX_FX: usize = 6;
    pub const MAX_SPATIAL_FX: usize = 3;
    pub const MAX_COLOR_FX: usize = 3;
    pub const MAX_VFX_ENTITIES: usize = 500;
    pub use crate::broadcast_material::*;
}

// TODO: re-export user stuff here
pub mod user {
    pub use crate::{VfxPlugin, render::Vfx, vfx::*};
}
