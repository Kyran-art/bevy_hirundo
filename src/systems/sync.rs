use crate::internal_prelude::*;

/// System to sync user-facing Vfx component to internal SpriteIndex component
pub fn sync_vfx_to_internal(mut query: Query<(&Vfx, &mut SpriteIndex), Changed<Vfx>>) {
    for (vfx, mut internal_sprite) in &mut query {
        internal_sprite.0 = vfx.sprite_index;
    }
}
