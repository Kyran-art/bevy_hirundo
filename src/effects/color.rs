use crate::internal_prelude::*;
use super::phase::Phase;
use super::wave::Wave;
use super::builder::{EffectBuilder, EffectModifier, LastEffect};

/// RGB color effect with wave-driven parameters.
///
/// Separated from alpha to allow independent control of color and transparency.
///
/// # Blend Modes (blend_mode)
/// - **0: Lerp** - Smooth interpolation between base and target color
/// - **1: Add** - Additive blending (brightens, good for glows/flashes)
/// - **2: Multiply** - Multiplicative blending (darkens, good for shadows)
/// - **3: Screen** - Inverse multiply (brightens without overexposure)
/// - **4: HSV Shift** - Hue/Saturation/Value manipulation
///
/// # Examples
///
/// **Color flash (additive)**
/// ```rust
/// ColorEffect {
///     phase: Phase::full(),
///     wave: Wave::sine(1.0, 0.5).with_bias(0.5),
///     color: LinearRgba::from(RED).to_vec3(),
///     blend_mode: 1, // Additive blend
/// }
/// ```
///
/// **HSV hue rotation**
/// ```rust
/// ColorEffect {
///     phase: Phase::full(),
///     wave: Wave::sine(1.0, 0.5).with_bias(0.5),
///     color: Vec3::new(
///         1.0,  // H: Full hue rotation (360 degrees)
///         0.0,  // S: No saturation change
///         0.0,  // V: No brightness change
///     ),
///     blend_mode: 4, // HSV shift
/// }
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, Default)]
pub struct ColorEffect {
    pub phase: Phase,
    pub wave: Wave,
    /// RGB or HSV color - use `.to_vec3()` for LinearRgba.
    ///
    /// **Important** the 4th value, usually reserved for Alpha, is repurposed as a ... flag.
    /// Alpha is controlled separately.
    pub color: Vec4,
    /// Blend mode: 0=Lerp, 1=Add, 2=Multiply, 3=Screen, 4=HSV
    pub blend_mode: u32,
}

impl ColorEffect {
    /// New RGB effect with a full phase and lerp blend mode.
    pub fn new(color: Vec4, wave: Wave) -> Self {
        Self {
            wave,
            color,
            blend_mode: 0,
            ..default()
        }
    }

    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode as u32;
        self
    }

    // TODO
    // HSV helper
}

/// Set the blend mode for your color effect.
/// - **0: Lerp** - Smooth interpolation between base and target color
/// - **1: Add** - Additive blending (brightens, good for glows/flashes)
/// - **2: Multiply** - Multiplicative blending (darkens, good for negative statuses)
/// - **3: Screen** - Inverse multiply (brightens without overexposure)
/// - **4: HSV Shift** - Hue/Saturation/Value manipulation
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum BlendMode {
    /// - **0: Lerp** - Smooth interpolation between base and target color
    #[default]
    Lerp = 0,
    /// - **1: Add** - Additive blending (brightens, good for glows/flashes)
    Add = 1,
    /// - **2: Multiply** - Multiplicative blending (darkens, good for negative statuses)
    Multiply = 2,
    /// - **3: Screen** - Inverse multiply (brightens without overexposure)
    Screen = 3,
    /// - **4: HSV Shift** - Hue/Saturation/Value manipulation
    Hsv = 4,
}

impl EffectModifier for BlendMode {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().blend_mode = *self as u32
            }
            _ => warn!("No previous RGB effect to modify."),
        }
    }
}

/// How effect colors accumulate.
///
/// Defaults to **Contributive**.
#[derive(Clone, Copy, Debug, Default)]
#[repr(u32)]
pub enum CompositeMode {
    /// **(Mode 0: Sequential)**
    ///
    /// The effect is applied sequentially, meaning the output of the previous effect
    /// in the stack becomes the input for this effect.
    ///
    /// This mode typically leads to negatively compounding results,
    /// where an effect's final amplitude is *multiplied* by the amplitude of its predecessors.
    /// This mode is processed *after* all Accumulated (Contributive and Additive) effects.
    Multiplicative,

    /// **(Mode 1: Accumulated Blend)** -> **Default**
    ///
    /// The effect is accumulated with other Contributive and Additive effects in a pre-pass.
    ///
    /// All Contributive effects blend their color/hue values into a weighted average,
    /// but the final overall intensity (amplitude) of the accumulated color is
    /// **capped at the maximum intensity of the strongest single Contributive effect.**
    #[default]
    Contributive,

    /// **(Mode 2: Accumulated Sum)**
    ///
    /// The effect is accumulated with other Contributive and Additive effects in a pre-pass.
    ///
    /// Both the color channels (RGB) and the overall intensity (amplitude) of all
    /// Additive effects are **strictly summed** together. This mode is suitable for
    /// stacking light sources or damage flashes where intensity should be allowed to stack
    /// without a cap (bounded only by float maximum).
    Additive,
}

impl EffectModifier for CompositeMode {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(idx)) => {
                builder.colors[idx].as_mut().unwrap().color.w = *self as u32 as f32
                // smell
            }
            _ => warn!("No previous sub-effect to modify."),
        }
    }
}
