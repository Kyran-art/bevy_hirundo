use crate::internal_prelude::*;

pub fn dehydrate_vfx(mut world: DeferredWorld, context: HookContext) {
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
