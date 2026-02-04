use crate::internal_prelude::*;

pub fn hydrate_vfx(mut world: DeferredWorld, context: HookContext) {
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
