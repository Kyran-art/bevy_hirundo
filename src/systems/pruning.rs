use crate::internal_prelude::*;

/// System to prune expired effects (optional - keeps effect stacks clean)
pub fn prune_expired_effects(time: Res<Time>, mut query: Query<&mut Vfx>) {
    let now = time.elapsed_secs();
    for mut vfx in &mut query {
        vfx.effects.expire(now);
    }
}
