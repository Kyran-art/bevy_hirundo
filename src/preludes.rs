// Internal prelude for use within the crate
pub mod internal {
    pub use crate::components::*;
    pub use crate::effects::*;
    pub use crate::materials::*;
    pub use crate::resources::*;
    pub use crate::systems::*;

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

// User-facing prelude - minimal API surface
pub mod user {
    // Plugin
    pub use crate::HirundoPlugin;

    // Core components
    pub use crate::components::{Vfx, VfxBroadcast, VfxBundle};

    // Effects API (builders, modifiers, enums)
    pub use crate::effects::{
        AlphaEffect, Anchor, BlendMode, ColorEffect, CompositeMode, Effect, EffectBuilder,
        EffectModifier, EffectStack, Envelope, Lifetime, Phase, SpatialEffect, Wave, WaveKind,
    };

    // Resources (only what users might need to access)
    pub use crate::resources::{AtlasDimensions, VfxBroadcastMaterialHandle};

    // Optional: Broadcast update system (if users want manual control)
    pub use crate::systems::update_broadcast_effect_stack;

    // Optional: Demo input systems (for testing/examples)
    pub use crate::input::{control_broadcast_fx, control_unique_fx};

    // Spawner helpers (convenience functions)
    pub use crate::spawners::*;
}
