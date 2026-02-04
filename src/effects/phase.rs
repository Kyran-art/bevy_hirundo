use crate::internal_prelude::*;
use super::builder::{EffectBuilder, EffectModifier, LastEffect};

/// Sub-effect lifetime/window as a fraction of the overall Effect's lifetime.
///
/// [`Wave`] frequency is constant for any phase.
///
/// # Example
/// ```
/// EffectBuilder::one_shot(now, 1.0) // effect lifetime is 2 seconds
/// .offset_x(10)
/// .with(Phase::new(0.2, 0.8)) // offset_x starts at 0.4 secs (20% of 2 seconds), ends at 1.6 secs.
/// ```
#[repr(C)]
#[derive(Clone, Copy, ShaderType, Debug, PartialEq)]
pub struct Phase {
    /// Start time as fraction of master duration (0.0 to 1.0)
    pub start: f32,
    /// End time as fraction of master duration (0.0 to 1.0)
    pub end: f32,
    /// Padding to ensure 16-byte alignment
    _padding: Vec2,
}

impl Default for Phase {
    fn default() -> Self {
        Self::full()
    }
}

impl Phase {
    pub fn new(start: f32, end: f32) -> Self {
        Self {
            start,
            end,
            _padding: Vec2::ZERO,
        }
    }
    pub fn full() -> Self {
        Self {
            start: 0.0,
            end: 1.0,
            _padding: Vec2::ZERO,
        }
    }
    pub fn first_half() -> Self {
        Self {
            start: 0.0,
            end: 0.5,
            _padding: Vec2::ZERO,
        }
    }
    pub fn second_half() -> Self {
        Self {
            start: 0.5,
            end: 1.0,
            _padding: Vec2::ZERO,
        }
    }

    pub fn start(time: f32) -> Self {
        Self {
            start: time,
            end: 1.0,
            _padding: Vec2::ZERO,
        }
    }

    pub fn end(time: f32) -> Self {
        Self {
            start: 0.0,
            end: time,
            _padding: Vec2::ZERO,
        }
    }
}

impl EffectModifier for Phase {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => builder.colors[idx].as_mut().unwrap().phase = *self,
            Some(LastEffect::Alpha) => builder.alpha.as_mut().unwrap().phase = *self,
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().phase = *self
            }
            None => warn!("No previous sub-effect to modify."),
        }
    }
}
