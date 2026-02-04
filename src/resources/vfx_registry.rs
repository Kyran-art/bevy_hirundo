use crate::internal_prelude::*;

#[derive(Resource, Default)]
pub struct VfxRegistry {
    // Maps a hash of an EffectStack to a specific buffer index
    pub active_effects: HashMap<u64, u32>,
    pub slot_ref_counts: Vec<usize>, // Track how many entities use each slot
}
