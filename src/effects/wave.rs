use super::builder::{EffectBuilder, EffectModifier, LastEffect};
use super::envelope::Envelope;
use crate::internal_prelude::*;

/// The **Constant** wave is the default for most [`EffectBuilder`] sub-effects.
///
/// This is an [`EffectModifier`].
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum WaveKind {
    /// S
    Sine = 0,
    /// __ -- __ --
    Square = 1,
    /// / \ / \
    Triangle = 2,
    /// /| /|
    Saw = 3,
    /// ————————
    #[default]
    Constant = 4,
}

impl EffectModifier for WaveKind {
    /// Update the `kind` of the most recent sub-effect's wave.
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.kind = *self as u32
            }
            Some(LastEffect::Alpha) => builder.alpha.as_mut().unwrap().wave.kind = *self as u32,
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.kind = *self as u32
            }
            None => warn!("No previous sub-effect to modify."),
        }
    }
}
/// Wave parameters for oscillation, ramp or constancy.
///
/// Oscillating waves (sine and triangle) use a cosine-based phase convention,
/// meaning they begin at their **Peak** (+1.0) when phase is 0.0,
/// unlike a standard sine function which begins at the center (0.0).
/// The inverse is true for ramping waves.
///
/// # Fields
/// - `kind`: 0=sin, 1=square, 2=triangle, 3=saw
/// - `freq`: Cycles per effect duration (0.5 = half cycle, 1.0 = full cycle)
/// - `amp`: Wave amplitude (peak-to-trough distance) (sign determines starting direction)
/// - `bias`: Center point offset
/// - `phase`: Starting point
/// - `amp_envelope`: Envelope controlling amplitude modulation over time
/// - `freq_envelope`: Envelope controlling frequency modulation over time
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, PartialEq, Default)]
pub struct Wave {
    /// 0=sin, 1=square, 2=triangle, 3=saw, 4=constant
    pub(crate) kind: u32,
    pub(crate) freq: f32,
    pub(crate) amp: f32,
    pub(crate) bias: f32,
    pub(crate) phase: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    pub(crate) amp_envelope: Envelope,  // 32 bytes
    pub(crate) freq_envelope: Envelope, // 32 bytes
}

impl Wave {
    pub fn new(kind: u32, freq: f32, amp: f32, bias: f32, phase: f32) -> Self {
        Self {
            kind,
            freq,
            amp,
            bias,
            phase,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
        }
    }
    pub fn sine(freq: f32, amp: f32, bias: f32) -> Self {
        Self {
            kind: WaveKind::Sine as u32,
            freq,
            amp,
            bias,
            phase: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
        }
    }
    pub fn square(freq: f32, amp: f32, bias: f32) -> Self {
        Self {
            kind: WaveKind::Square as u32,
            freq,
            amp,
            bias,
            phase: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
        }
    }
    pub fn triangle(freq: f32, amp: f32, bias: f32) -> Self {
        Self {
            kind: WaveKind::Triangle as u32,
            freq,
            amp,
            bias,
            phase: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
        }
    }
    pub fn saw(freq: f32, amp: f32, bias: f32) -> Self {
        Self {
            kind: WaveKind::Saw as u32,
            freq,
            amp,
            bias,
            phase: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
        }
    }

    /// Creates a constant value wave (no oscillation).
    ///
    /// Useful for static offsets, scales, or colors that should remain constant
    /// but still benefit from envelope modulation.
    ///
    /// The `value` parameter sets the constant output (typically combined with bias).
    pub fn constant(value: f32) -> Self {
        Self {
            kind: WaveKind::Constant as u32,
            freq: 0.0,
            amp: value,
            bias: 0.0,
            phase: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
        }
    }

    /// This **must be called first** before any other `with_amp_envelope_...` methods.
    pub fn with_amp_envelope(mut self, attack: f32, hold: f32, release: f32) -> Self {
        self.amp_envelope = Envelope::new(attack, hold, release);
        self
    }

    /// Sets the growth mode of the amplitude envelope to exponential.
    ///
    /// **Precondition:** Must be called after `.with_amp_envelope(attack, hold, release)`
    /// to ensure the envelope is initialized.
    ///
    /// Applies exponential curvature to the attack/attack phase only.
    ///
    /// **Range**: -1.0 (smooth attack) to 1.0 (aggressive attack)
    pub fn with_amp_envelope_exponential_growth(mut self, strength: f32) -> Self {
        self.amp_envelope = self.amp_envelope.with_ease_in(strength);
        self
    }

    /// Sets the decay mode of the amplitude envelope to exponential.
    ///
    /// **Precondition:** Must be called after `.with_amp_envelope(attack, hold, release)`
    /// to ensure the envelope is initialized.
    ///
    /// Applies exponential curvature to the release/release phase only.
    ///
    /// **Range**: -1.0 (gradual fade) to 1.0 (quick fade)
    pub fn with_amp_envelope_exponential_decay(mut self, strength: f32) -> Self {
        self.amp_envelope = self.amp_envelope.with_ease_out(strength);
        self
    }

    /// Sets the Attack-Hold-Decay (AHD) parameters for the frequency envelope.
    /// This **must be called first** before any other `with_freq_envelope_...` methods.
    pub fn with_freq_envelope(mut self, attack: f32, hold: f32, release: f32) -> Self {
        self.freq_envelope = Envelope::new(attack, hold, release);
        self
    }

    /// Sets the growth mode of the frequency envelope to exponential.
    ///
    /// **Precondition:** Must be called after `.with_freq_envelope(attack, hold, release)`
    /// to ensure the envelope is initialized.
    ///
    /// Applies exponential curvature to the attack/attack phase only.
    ///
    /// **Range**: -1.0 (smooth ramp-up) to 1.0 (aggressive ramp-up)
    pub fn with_freq_envelope_exponential_growth(mut self, strength: f32) -> Self {
        self.freq_envelope = self.freq_envelope.with_ease_in(strength);
        self
    }

    /// Sets the decay mode of the frequency envelope to exponential.
    ///
    /// **Precondition:** Must be called after `.with_freq_envelope(attack, hold, release)`
    /// to ensure the envelope is initialized.
    ///
    /// Applies exponential curvature to the release/release phase only.
    ///
    /// **Range**: -1.0 (gradual slow-down) to 1.0 (quick slow-down)
    pub fn with_freq_envelope_exponential_decay(mut self, strength: f32) -> Self {
        self.freq_envelope = self.freq_envelope.with_ease_out(strength);
        self
    }
    pub fn with_bias(mut self, bias: f32) -> Self {
        self.bias = bias;
        self
    }
    pub fn with_kind(mut self, kind: u32) -> Self {
        self.kind = kind;
        self
    }
    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }
    /// The wave begins from its bias.
    ///
    /// Good for spatial movements that occur around a sprite's original position.
    pub fn with_center_phase(mut self) -> Self {
        let center: f32 = match self.kind {
            0 | 2 => 0.25,
            1 | 3 => 0.5,
            _ => 0.25,
        };
        self.phase = center;
        self
    }
    /// Identical to *with_center_phase* with no return.
    pub fn center_phase(&mut self) {
        let center: f32 = match self.kind {
            0 | 2 => 0.25,
            1 | 3 => 0.5,
            _ => 0.25,
        };
        self.phase = center;
    }

    pub fn rotate_oscillating(freq: f32, degrees: f32) -> Self {
        let rad = degrees.to_radians() / 2.0;
        Self {
            kind: WaveKind::Sine as u32,
            freq,
            amp: rad,
            bias: rad,
            phase: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
            ..default()
        }
    }
    pub fn rotate_continuous(freq: f32, degrees: f32) -> Self {
        let rad = degrees.to_radians() / 2.0;
        Self {
            kind: WaveKind::Saw as u32,
            freq,
            amp: rad,
            bias: rad,
            phase: 0.0,
            amp_envelope: Envelope::disabled(),
            freq_envelope: Envelope::disabled(),
            ..default()
        }
    }
}

impl EffectModifier for Wave {
    /// Replaces the wave of the last sub-effect added.
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => builder.colors[idx].as_mut().unwrap().wave = *self,
            Some(LastEffect::Alpha) => builder.alpha.as_mut().unwrap().wave = *self,
            Some(LastEffect::Spatial(kind)) => builder.spatial[kind].as_mut().unwrap().wave = *self,
            None => warn!("No previous sub-effect to modify."),
        }
    }
}

/// Where the wave begins.
///
/// Oscillating waves (sine and triangle) use a cosine-based phase convention,
/// meaning they begin at their **Peak** (+1.0) when phase is 0.0,
/// unlike a standard sine function which begins at the center (0.0).
/// Negative amplitude reverses this to their **Trough** (-1.0).
///
/// The inverse is true for ramping waves.
#[derive(Clone, Copy, From)]
pub struct WavePhase(pub f32);
impl WavePhase {
    // Custom constructor for the best ergonomics:
    pub fn new(phase: f32) -> Self {
        phase.into() // Uses the auto-derived 'From<f32>'
    }
    /// The wave begins from its bias.
    ///
    /// Good for spatial movements oscillating around a sprite's original position.
    pub fn center() -> WavePhaseCenter {
        WavePhaseCenter
    }
}
impl EffectModifier for WavePhase {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.phase = self.0;
            }
            Some(LastEffect::Alpha) => {
                builder.alpha.as_mut().unwrap().wave.phase = self.0;
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.phase = self.0;
            }
            None => warn!("Cannot apply WavePhase: No previous effect to modify."),
        }
    }
}

/// Helper for WavePhase.
#[derive(Clone, Copy)]
pub struct WavePhaseCenter;
impl EffectModifier for WavePhaseCenter {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.center_phase()
            }
            Some(LastEffect::Alpha) => builder.alpha.as_mut().unwrap().wave.center_phase(),
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.center_phase()
            }
            None => warn!("No previous sub-effect to modify."),
        }
    }
}
#[derive(Clone, Copy, From)]
pub struct Bias(pub f32);
impl Bias {
    pub fn new(bias: f32) -> Self {
        bias.into()
    }
}
impl EffectModifier for Bias {
    #[doc(hidden)]
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.bias = self.0;
            }
            Some(LastEffect::Alpha) => {
                builder.alpha.as_mut().unwrap().wave.bias = self.0;
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.bias = self.0;
            }
            None => warn!("Cannot apply Amplitude: No previous effect to modify."),
        }
    }
}

/// Update the amplitude of the most recent sub-effect's wave.
///
/// Amplitude applies differently across domains:
///
/// **Color**
///
/// *rgb* -> fraction (0.0 = 0%, 1.0 = 100%),
///
/// *hsv* -> degrees
///
/// **Space**
///
/// *offset* -> pixels
///
/// *scale* -> factor (1.0 = 100% scale)
///
/// *skew* -> factor (1.0 = 45 degrees)
///
/// *rotate* -> degrees
#[derive(Clone, Copy, From)]
pub struct Amplitude(pub f32);
impl EffectModifier for Amplitude {
    #[doc(hidden)]
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.amp = self.0;
            }
            Some(LastEffect::Alpha) => {
                builder.alpha.as_mut().unwrap().wave.amp = self.0;
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.amp = self.0;
            }
            None => warn!("Cannot apply Amplitude: No previous effect to modify."),
        }
    }
}
/// Update the frequency of the most recent sub-effect's wave.
/// **note**: The *constant* wave, for which all sub-effects are initialized, is the only *wave kind* unaffected by Frequency.
#[derive(Clone, Copy, From)]
pub struct Frequency(pub f32);
impl EffectModifier for Frequency {
    #[doc(hidden)]
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.freq = self.0;
            }
            Some(LastEffect::Alpha) => {
                builder.alpha.as_mut().unwrap().wave.freq = self.0;
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.freq = self.0;
            }
            None => warn!("Cannot apply Frequency: No previous effect to modify."),
        }
    }
}
