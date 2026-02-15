use crate::internal_prelude::*;

/// `Vfx` is designed to be added once and kept for the lifetime of the entity.
/// Repeatedly removing and re-adding `Vfx` (or the required bundle components)
/// causes archetype thrashing in Bevy's ECS, leading to progressively worse
/// performance (increasing lag) over time due to table fragmentation and
/// column reallocation overhead.
///
/// This is a known limitation of the current implementation.
///
/// **Preferred patterns**:
///
/// - For temporary/one-shot effects: spawn a new entity, push effects, then despawn when done.
/// - For persistent effects on game objects: add `Vfx` once at spawn and keep it forever.
///   Toggle visibility by clearing effects and/or switching to a blank sprite.
/// - To "hide" without despawning: use `vfx.hide()` (see below) or push a looping
///   effect that sets scale = 0.0 or alpha = 0.0.
///
/// Removing the component is allowed but strongly discouraged for performance-critical use.
/// If you must remove `Vfx`, remove `VfxBundle` to mitigate archetype thrashing.
/// Removing `Vfx` alone will leave behind the other components added by `#[require]`.
#[derive(Component)]
#[component(on_add = crate::hooks::hydrate_vfx, on_remove = crate::hooks::dehydrate_vfx)]
#[require(MeshTag, Mesh2d, MeshMaterial2d<VfxMaterial>, SpriteIndex, Visibility, VfxGhostBuffer)]
pub struct Vfx {
    pub sprite_index: u32,
    pub(crate) effects: EffectStack,
}

impl Vfx {
    pub fn with_sprite(sprite_index: u32) -> Self {
        Vfx {
            sprite_index,
            effects: EffectStack::default(),
        }
    }

    pub fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn clear_effects(&mut self) {
        self.effects.clear();
    }
}

impl Default for Vfx {
    fn default() -> Self {
        Self::with_sprite(0)
    }
}

/// Bundle including all required components for `Vfx` to function.
/// Use this to remove `Vfx` without leaving behind orphaned components.
/// Although, its use is strongly discouraged due to archetype thrashing issues.
///
/// Prefer to despawn entities with `Vfx`, or use `Visibility` rather than removing the component/s.
#[derive(Bundle)]
pub struct VfxBundle {
    pub vfx: Vfx,
    pub tag: MeshTag,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<VfxMaterial>,
    pub sprite_index: SpriteIndex,
    pub effects: EffectStack, // Helper component for stack logic
}

impl Default for VfxBundle {
    fn default() -> Self {
        VfxBundle {
            vfx: Vfx::default(),
            tag: MeshTag(0),
            mesh: Mesh2d(Handle::default()),
            material: MeshMaterial2d(Handle::default()),
            sprite_index: SpriteIndex(0),
            effects: EffectStack::default(),
        }
    }
}
