use crate::internal_prelude::*;

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
