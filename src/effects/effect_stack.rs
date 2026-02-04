use crate::internal_prelude::*;
use super::lifetime::Lifetime;
use super::color::ColorEffect;
use super::alpha::AlphaEffect;
use super::spatial::SpatialEffect;

/// Complete effect containing master timing and sub-effects.
/// RGB and Alpha are now separate for independent control.
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, Default)]
pub struct Effect {
    pub(crate) lifetime: Lifetime,
    pub(crate) color_effects: [ColorEffect; MAX_COLOR_FX],
    pub(crate) alpha_effect: AlphaEffect,
    pub(crate) spatial_effects: [SpatialEffect; MAX_SPATIAL_FX],
}

impl Effect {
    /// Creates a new, empty effect, ready for building.
    pub fn new_one_shot(now: f32, duration: f32) -> Self {
        Self {
            lifetime: Lifetime::one_shot(now, duration),
            ..default()
        }
    }

    /// Creates a new, empty effect, ready for building.
    pub fn new_looping(now: f32, period: f32) -> Self {
        Self {
            lifetime: Lifetime::looping(now, period),
            ..default()
        }
    }
}

/// Stack of up to MAX_FX simultaneous effects.
#[repr(C)]
#[derive(Component, Clone, ShaderType, Debug, Default)]
pub struct EffectStack {
    pub tile_index: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
    pub effects: [Effect; MAX_FX],
}

impl EffectStack {
    pub fn clear(&mut self) {
        for eff in &mut self.effects {
            eff.lifetime.enabled = 0;
        }
    }

    /// Use a disabled slot or overwrite the oldest
    pub fn push(&mut self, effect: Effect) {
        for slot in &mut self.effects {
            if slot.lifetime.enabled == 0 {
                *slot = effect;
                return;
            }
        }
        self.effects[0] = effect;
    }

    /// Disable expired one-shot effects
    pub fn expire(&mut self, now: f32) {
        for eff in &mut self.effects {
            let t = eff.lifetime;
            if t.enabled == 1 && t.looping == 0 && now >= t.start_time + t.duration {
                eff.lifetime.enabled = 0;
            }
        }
    }
}
