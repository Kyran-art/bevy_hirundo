use crate::internal_prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    platform::collections::HashMap,
};
use std::collections::{HashSet, VecDeque};

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
#[component(on_add = hydrate_vfx, on_remove = dehydrate_vfx)]
#[require(MeshTag, Mesh2d, MeshMaterial2d<VfxMaterial>, SpriteIndex, Visibility, VfxGhostBuffer)]
pub struct Vfx {
    pub sprite_index: u32,
    pub(crate) effects: EffectStack,
}

impl Vfx {
    pub fn new(sprite_index: u32) -> Self {
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
        Self::new(0)
    }
}

/// Marker component to delay visibility, veiling ghost sprites during initialization.
#[derive(Component, Default)]
pub struct VfxGhostBuffer;

#[derive(Resource, Default)]
pub struct VfxRegistry {
    // Maps a hash of an EffectStack to a specific buffer index
    pub active_effects: HashMap<u64, u32>,
    pub slot_ref_counts: Vec<usize>, // Track how many entities use each slot
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

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SpriteIndex(pub u32);

#[derive(Resource, FromWorld)]
pub struct MeshTagAllocator {
    pub next_tag: u32,
    pub free_list: VecDeque<u32>,
}

impl MeshTagAllocator {
    pub fn new() -> Self {
        MeshTagAllocator {
            next_tag: 0,
            free_list: VecDeque::new(),
        }
    }

    pub fn allocate_tag(&mut self) -> MeshTag {
        if let Some(recycled_tag) = self.free_list.pop_front() {
            MeshTag(recycled_tag)
        } else {
            let tag = self.next_tag;
            self.next_tag += 1;
            MeshTag(tag)
        }
    }

    pub fn free_tag(&mut self, tag: u32) {
        self.free_list.push_back(tag);
    }
}

fn hydrate_vfx(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    // 1. Get our dynamic data
    let tag = world.resource_mut::<MeshTagAllocator>().allocate_tag();
    let mesh = world.resource::<VfxMeshHandle>().0.clone();
    let mat = world.resource::<VfxMaterialHandle>().0.clone();
    let sprite_val = world
        .get::<Vfx>(entity)
        .map(|v| v.sprite_index)
        .unwrap_or(0);

    // 2. Mark slot as dirty
    world
        .resource_mut::<EffectStorageData>()
        .dirty_slots
        .insert(tag.0 as usize);

    world.commands().entity(entity).insert(VfxGhostBuffer);

    // 3. MODIFY the components that were just added by #[require]
    // These calls are legal in DeferredWorld because they don't change the archetype!
    if let Some(mut vis) = world.get_mut::<Visibility>(entity) {
        *vis = Visibility::Hidden;
    }
    if let Some(mut tag_comp) = world.get_mut::<MeshTag>(entity) {
        *tag_comp = tag; // 2. Overwrites whatever was there
    }
    if let Some(mut m) = world.get_mut::<Mesh2d>(entity) {
        m.0 = mesh;
    }
    if let Some(mut mat_comp) = world.get_mut::<MeshMaterial2d<VfxMaterial>>(entity) {
        mat_comp.0 = mat;
    }
    if let Some(mut s) = world.get_mut::<SpriteIndex>(entity) {
        s.0 = sprite_val;
    }
}

fn dehydrate_vfx(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    // 1. Recycle the ID and Clear GPU Slot
    // We can still access the MeshTag here because the removal command
    // hasn't fully applied the archetype change yet.
    if let Some(tag) = world.get::<MeshTag>(entity).map(|t| t.0) {
        let mut storage = world.resource_mut::<EffectStorageData>();
        if let Some(slot) = storage.effects.get_mut(tag as usize) {
            slot.clear();
            slot.tile_index = 0;
            // Mark dirty so the GPU buffer updates ONCE
            storage.dirty_slots.insert(tag as usize);
        }

        world.resource_mut::<MeshTagAllocator>().free_tag(tag);
        info!("Dehydrate → recycled tag {}", tag);
    }

    // 2. STOP. Do not call commands().remove() here.
}

#[derive(Resource, Deref, DerefMut)]
pub struct VfxMaterialHandle(pub Handle<VfxMaterial>);

#[derive(Resource, Deref, DerefMut)]
pub struct VfxMeshHandle(pub Handle<Mesh>);

#[derive(Resource)]
pub struct EffectStorageData {
    pub effects: Vec<EffectStack>,
    pub dirty_slots: HashSet<usize>,
}

impl FromWorld for EffectStorageData {
    fn from_world(_world: &mut World) -> Self {
        Self {
            effects: vec![EffectStack::default(); MAX_VFX_ENTITIES],
            dirty_slots: HashSet::new(),
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
pub struct AtlasDimensions {
    pub texture_size: Vec2,
    pub cell_size: Vec2,
    pub sprite_size: Vec2,
    pub padding: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VfxMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[storage(2, read_only)]
    pub effect_storage: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub atlas_dimensions: AtlasDimensions,
}

impl Material2d for VfxMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx.wgsl".into()
    }
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx.wgsl".into()
    }
    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
