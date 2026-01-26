pub mod camera;
mod preludes;
pub mod render;
pub mod systems;
pub mod vfx;

pub use crate::preludes::internal as internal_prelude;
pub use crate::preludes::user as prelude;

use crate::internal_prelude::*;

#[derive(Resource)]
pub struct VfxPlugin {
    pub texture_path: String,
    pub atlas_dimensions: AtlasDimensions,
}

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VfxPlugin {
            texture_path: self.texture_path.clone(),
            atlas_dimensions: self.atlas_dimensions.clone(),
        });
        app.init_resource::<MeshTagAllocator>();
        app.init_resource::<EffectStorageData>();
        app.init_asset::<ShaderStorageBuffer>();

        app.insert_resource(VfxMeshHandle(Handle::default()));
        app.insert_resource(VfxMaterialHandle(Handle::default()));

        app.add_plugins(Material2dPlugin::<VfxMaterial>::default())
            .add_systems(
                Update,
                (
                    sync_vfx_to_internal,
                    update_effect_storage_buffer,
                    prune_expired_effects,
                )
                    .chain(),
            );
        app.add_systems(PreStartup, setup_vfx_assets);
    }
}

impl Default for VfxPlugin {
    fn default() -> Self {
        VfxPlugin {
            texture_path: "32roguesTextureV2.png".to_string(),
            atlas_dimensions: AtlasDimensions {
                texture_size: Vec2::new(1024.0, 1024.0),
                cell_size: Vec2::new(40.0, 40.0),
                sprite_size: Vec2::new(32.0, 32.0),
                padding: Vec2::new(4.0, 4.0),
            },
        }
    }
}

impl VfxPlugin {
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
