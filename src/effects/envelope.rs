use crate::internal_prelude::*;
use super::builder::{EffectBuilder, EffectModifier, LastEffect};

/// Growth mode for envelope amplitude modulation
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum GrowthMode {
    #[default]
    None = 0, // No growth applied (passthrough)
    Exponential = 1, // Exponential growth (e^x style)
}

/// Parameters for wave modulation over time.
///
/// - **Attack**: Time to rise from 0 to target amplitude/frequency
/// - **Hold**: Time sustained at target amplitude/frequency
/// - **Release**: Time to fall from target to 0 amplitude/frequency
///
/// attack + hold + release must sum to 1.0 and are fractions of [`Phase`]
///
/// ```rust
/// EffectBuilder::one_shot(time.elapsed_secs(), 1.0)
/// .skew_x(0.4) // 0.4 is target amplitude
/// .with(Envelope::amplitude(0.2, 0.0, 0.8)) // 0 to target in 0.2 seconds, target to 0 in 0.8
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, Default, PartialEq)]
pub struct Envelope {
    /// Rise time as fraction of phase (0.0 to 1.0)
    attack: f32,
    /// Hold time at wave peak as fraction of phase (0.0 to 1.0)
    hold: f32,
    /// Fall time as fraction of phase (0.0 to 1.0)
    release: f32,
    /// Growth mode for attack/attack (0=none, 1=exponential)
    growth_mode: u32,
    /// Growth factor/strength for attack/attack
    growth: f32,
    /// Enable flag: 0=disabled (passthrough), 1=enabled
    enabled: u32,
    /// Decay mode for release/release (0=none, 1=exponential)
    decay_mode: u32,
    /// Decay factor/strength for release/release
    decay: f32,
}
impl Envelope {
    // === Effect Modifiers ===

    /// Returns an [`EffectModifier`] that modulates the most recent sub-effect wave's amplitude.
    pub fn amplitude(attack: f32, hold: f32, release: f32) -> AmplitudeEnvelope {
        AmplitudeEnvelope(Self::new(attack, hold, release))
    }

    /// Returns an [`EffectModifier`] that modulates the most recent sub-effect wave's frequency.
    pub fn frequency(attack: f32, hold: f32, release: f32) -> FrequencyEnvelope {
        FrequencyEnvelope(Self::new(attack, hold, release))
    }

    // ===

    /// Create a new envelope with specified timings
    pub(crate) fn new(attack: f32, hold: f32, release: f32) -> Self {
        Self {
            attack,
            hold,
            release,
            growth_mode: GrowthMode::None as u32,
            growth: 0.0,
            enabled: 1,
            decay_mode: GrowthMode::None as u32,
            decay: 0.0,
        }
    }

    /// Disabled envelope - passthrough (no envelope applied)
    pub(crate) fn disabled() -> Self {
        Self {
            attack: 0.0,
            hold: 0.0,
            release: 0.0,
            growth_mode: GrowthMode::None as u32,
            growth: 0.0,
            enabled: 0,
            decay_mode: GrowthMode::None as u32,
            decay: 0.0,
        }
    }

    // === self Modifiers ===

    /// Exponentially curve the attack. (The attack starts slower but quickly accelerates)
    pub fn with_ease_in(mut self, strength: f32) -> Self {
        self.growth_mode = GrowthMode::Exponential as u32;
        self.growth = strength;
        self
    }

    /// Exponentially curve the release. (The release starts faster but quickly decelerates)
    pub fn with_ease_out(mut self, strength: f32) -> Self {
        self.decay_mode = GrowthMode::Exponential as u32;
        self.decay = -strength;
        self
    }
}

/// For future [`EffectBuilder`]/[`EffectModifier`] helpers i.e. *FadeIn*
#[derive(Clone, Copy, Debug, Default)]
struct EnvelopeIntent {
    attack: Option<f32>,
    hold: Option<f32>,
    release: Option<f32>,
    explicit: Option<Envelope>,
}

/// Newtype wrapper for [`Envelope`], explicitly targets the Wave's amplitude envelope.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, From)]
pub struct AmplitudeEnvelope(pub Envelope);
impl AmplitudeEnvelope {
    /// Constructs a new AmplitudeEnvelope, leveraging the inner Envelope's builder.
    pub fn new(attack: f32, hold: f32, release: f32) -> Self {
        Envelope::new(attack, hold, release).into()
    }

    /// Exponentially curve the attack. (The attack starts slower but quickly accelerates)
    pub fn with_ease_in(mut self, strength: f32) -> Self {
        self = Self(self.0.with_ease_in(strength));
        self
    }

    /// Exponentially curve the release. (The release starts faster but quickly decelerates)
    pub fn with_ease_out(mut self, strength: f32) -> Self {
        self = Self(self.0.with_ease_out(strength));
        self
    }
}
impl EffectModifier for AmplitudeEnvelope {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.amp_envelope = self.0;
            }
            Some(LastEffect::Alpha) => {
                builder.alpha.as_mut().unwrap().wave.amp_envelope = self.0;
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.amp_envelope = self.0;
            }
            None => {
                warn!(
                    "Cannot apply AmplitudeEnvelope: No previous color or spatial effect to modify."
                )
            }
        }
    }
}

/// Newtype wrapper for [`Envelope`], explicitly targets the Wave's frequency envelope.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, From)]
pub struct FrequencyEnvelope(pub Envelope);
impl FrequencyEnvelope {
    /// Constructs a new FreqEnvelope, leveraging the inner Envelope's builder.
    pub fn new(attack: f32, hold: f32, release: f32) -> Self {
        Envelope::new(attack, hold, release).into()
    }

    /// Exponentially curve the attack. (The attack starts slower but quickly accelerates)
    pub fn with_ease_in(mut self, strength: f32) -> Self {
        self = Self(self.0.with_ease_in(strength));
        self
    }

    /// Exponentially curve the release. (The release starts faster but quickly decelerates)
    pub fn with_ease_out(mut self, strength: f32) -> Self {
        self = Self(self.0.with_ease_out(strength));
        self
    }
}
impl EffectModifier for FrequencyEnvelope {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().wave.freq_envelope = self.0;
            }
            Some(LastEffect::Alpha) => {
                builder.alpha.as_mut().unwrap().wave.freq_envelope = self.0;
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().wave.freq_envelope = self.0;
            }
            None => {
                warn!("Cannot apply FreqEnvelope: No previous color or spatial effect to modify.")
            }
        }
    }
}
