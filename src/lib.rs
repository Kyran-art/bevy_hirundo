// New module structure
pub mod components;
pub mod effects;
pub mod hooks;
pub mod input;
pub mod materials;
mod preludes;
pub mod resources;
pub mod spawners;
pub mod systems;

// Internal prelude is truly private - users should never need it
use crate::preludes::internal as internal_prelude;
pub use crate::preludes::user as prelude;

use crate::internal_prelude::*;

#[derive(Resource)]
pub struct HirundoPlugin {
    pub texture_path: String,
    pub atlas_dimensions: AtlasDimensions,
    pub with_camera: bool,
}

impl Plugin for HirundoPlugin {
    fn build(&self, app: &mut App) {
        // Store config as resource
        app.insert_resource(HirundoPlugin {
            texture_path: self.texture_path.clone(),
            atlas_dimensions: self.atlas_dimensions.clone(),
            with_camera: self.with_camera,
        });

        // Core resources
        app.init_resource::<MeshTagAllocator>();
        app.init_resource::<EffectStorageData>();
        app.init_asset::<ShaderStorageBuffer>();
        app.insert_resource(VfxMeshHandle(Handle::default()));
        app.insert_resource(VfxMaterialHandle(Handle::default()));

        // Per-entity VFX material (unique effects)
        app.add_plugins(Material2dPlugin::<VfxMaterial>::default());
        app.add_systems(PreStartup, setup_vfx_assets);
        app.add_systems(
            Update,
            (
                sync_vfx_to_internal,
                update_effect_storage_buffer,
                prune_expired_effects,
            )
                .chain(),
        );

        // Broadcast VFX material (shared effects) - always available
        app.add_plugins(Material2dPlugin::<VfxBroadcastMaterial>::default());
        app.add_systems(PreStartup, setup_broadcast_material);

        // Optional: Camera spawn and controls
        if self.with_camera {
            app.add_systems(Startup, spawn_camera);
            app.add_systems(Update, control_2d_camera);
        }
    }
}

impl Default for HirundoPlugin {
    fn default() -> Self {
        HirundoPlugin {
            texture_path: "32roguesTextureV2.png".to_string(),
            atlas_dimensions: AtlasDimensions {
                texture_size: Vec2::new(1024.0, 1024.0),
                cell_size: Vec2::new(40.0, 40.0),
                sprite_size: Vec2::new(32.0, 32.0),
                padding: Vec2::new(4.0, 4.0),
            },
            with_camera: false,
        }
    }
}

impl HirundoPlugin {
    /// Enable automatic 2D camera spawn with WASD/ZX controls
    pub fn with_camera(mut self) -> Self {
        self.with_camera = true;
        self
    }

    pub fn with_texture(mut self, path: &str) -> Self {
        self.texture_path = path.to_string();
        self
    }

    pub fn with_atlas(mut self, atlas: AtlasDimensions) -> Self {
        self.atlas_dimensions = atlas;
        self
    }

    pub fn with_cell_size(mut self, size: Vec2) -> Self {
        self.atlas_dimensions.cell_size = size;
        self
    }

    pub fn with_sprite_size(mut self, size: Vec2) -> Self {
        self.atlas_dimensions.sprite_size = size;
        self
    }

    pub fn with_texture_size(mut self, size: Vec2) -> Self {
        self.atlas_dimensions.texture_size = size;
        self
    }

    pub fn with_padding(mut self, size: Vec2) -> Self {
        self.atlas_dimensions.padding = size;
        self
    }
}
