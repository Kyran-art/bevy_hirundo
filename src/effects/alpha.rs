use crate::internal_prelude::*;
use super::phase::Phase;
use super::wave::Wave;

/// Alpha effect with wave-driven parameters.
///
/// Controls sprite transparency independently from color.
///
/// # Example
///
/// **Fade out**
/// ```rust
/// AlphaEffect {
///     phase: Phase::full(),
///     wave: Wave::sine(0.5, -0.5)
///         .with_bias(0.5)  // Start at 1.0 (opaque), end at 0.0 (transparent)
///         .with_amp_envelope(0.0, 0.0, 1.0), // Linear fade
/// }
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct AlphaEffect {
    pub(crate) phase: Phase,
    pub(crate) wave: Wave,
    target_alpha: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl AlphaEffect {
    /// New alpha effect with a full phase
    pub fn new(target: f32, wave: Wave) -> Self {
        Self {
            target_alpha: target,
            wave: wave,
            ..default()
        }
    }

    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.phase = phase;
        self
    }
}

impl Default for AlphaEffect {
    fn default() -> Self {
        Self {
            phase: Phase::full(),
            wave: Wave::constant(0.0), // strength=0 => no-op
            target_alpha: 1.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}
