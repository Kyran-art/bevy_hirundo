use crate::internal_prelude::*;

/// Controls when the effect is active and its overall duration.
#[repr(C)]
#[derive(Clone, Copy, Default, ShaderType, Debug)]
pub struct Lifetime {
    /// 0 = disabled, 1 = enabled
    pub enabled: u32,
    /// 0 = one-shot, 1 = looping
    pub looping: u32,
    pub start_time: f32,
    pub duration: f32,
}

impl Lifetime {
    pub fn toggle(&mut self) {
        self.enabled = 1 - self.enabled;
    }
    pub fn one_shot(now: f32, duration: f32) -> Self {
        Self {
            enabled: 1,
            looping: 0,
            start_time: now,
            duration,
        }
    }
    pub fn looping(now: f32, period: f32) -> Self {
        Self {
            enabled: 1,
            looping: 1,
            start_time: now,
            duration: period,
        }
    }
    pub fn disabled() -> Self {
        Self {
            enabled: 0,
            looping: 0,
            start_time: 0.0,
            duration: 0.0,
        }
    }
}

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
    phase: Phase,
    wave: Wave,
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

/// Spatial (vertex) manipulation types
#[repr(u32)]
#[derive(Clone, Copy, Debug, Enum)]
pub enum SpatialKind {
    OffsetX = 0,  // Horizontal translation (full sprite movement)
    OffsetY = 1,  // Vertical translation (full sprite movement)
    ScaleX = 2,   // Horizontal scale (1.0 = normal)
    ScaleY = 3,   // Vertical scale (1.0 = normal)
    Rotation = 4, // Rotation in radians
    SkewX = 5,    // Shear on the x axis
    SkewY = 6,    // Shear on the y axis
}

/// Anchor presets for common pivot points.
///
/// Typically used as an [`EffectModifier`]
///
/// An important control parameter, **Anchor** defines the fulcrum point
/// of rotation and how scaling and skewing look.
#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    Center,       // (0.5, 0.5)
    TopLeft,      // (0.0, 1.0)
    TopCenter,    // (0.5, 1.0)
    TopRight,     // (1.0, 1.0)
    CenterLeft,   // (0.0, 0.5)
    CenterRight,  // (1.0, 0.5)
    BottomLeft,   // (0.0, 0.0)
    BottomCenter, // (0.5, 0.0)
    BottomRight,  // (1.0, 0.0)
}

impl Anchor {
    pub fn to_vec2(self) -> Vec2 {
        match self {
            Anchor::Center => Vec2::new(0.5, 0.5),
            Anchor::TopLeft => Vec2::new(0.0, 1.0),
            Anchor::TopCenter => Vec2::new(0.5, 1.0),
            Anchor::TopRight => Vec2::new(1.0, 1.0),
            Anchor::CenterLeft => Vec2::new(0.0, 0.5),
            Anchor::CenterRight => Vec2::new(1.0, 0.5),
            Anchor::BottomLeft => Vec2::new(0.0, 0.0),
            Anchor::BottomCenter => Vec2::new(0.5, 0.0),
            Anchor::BottomRight => Vec2::new(1.0, 0.0),
        }
    }
}

impl EffectModifier for Anchor {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Color(_)) | Some(LastEffect::Alpha) => {
                warn!("Cannot apply anchorage to color or alpha effects.")
            }
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind]
                    .as_mut()
                    .unwrap()
                    .with_anchor(self.to_vec2());
            }
            None => warn!("No previous sub-effect to modify."),
        }
    }
}

/// Vertex manipulation effect with wave-driven parameters.
///
/// # Manipulation Types
/// - **OffsetX/Y**: Full sprite translation (shake, slide, bounce)
/// - **ScaleX/Y**: Stretch/squash
/// - **Rotation**: Rotate sprite (in radians)
/// - **Skew**: Shear sprite
///
/// # Examples
///
/// **Horizontal shake**
/// ```rust
/// SpatialEffect {
///     phase: Phase::full(),
///     wave: Wave::square(10.0, 2.0),  // Fast square wave
///     manipulation: SpatialKind::OffsetX as u32,
///     intensity: 1.0,  // 2 pixel shake range
/// }
/// ```
///
/// **Squash and stretch (hit feedback)**
/// ```rust
/// // Squash Y
/// SpatialEffect {
///     phase: Phase::new(0.0, 0.3),
///     wave: Wave::sine(1.0, -0.3),  // Compress to 70% height
///     manipulation: SpatialKind::ScaleY as u32,
///     intensity: 1.0,
/// }
/// // Stretch X (pairs with squash for skew effect)
/// SpatialEffect {
///     phase: Phase::new(0.0, 0.3),
///     wave: Wave::sine(1.0, 0.3),  // Expand to 130% width
///     manipulation: SpatialKind::ScaleX as u32,
///     intensity: 1.0,
/// }
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, Default, PartialEq)]
pub struct SpatialEffect {
    pub phase: Phase,
    pub wave: Wave,
    /// Manipulation type (see SpatialKind enum)
    pub manipulation: u32,
    /// Multiplier for effect strength
    pub intensity: f32,
    /// Pivot/Origin
    pub anchor: Vec2,
}

impl SpatialEffect {
    pub fn disabled() -> Self {
        Self::default()
    }
    pub fn offset_x(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::OffsetX as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn offset_y(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::OffsetY as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn scale_x(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::ScaleX as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn scale_y(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::ScaleY as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn rotation(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::Rotation as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn skew_x(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::SkewX as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn skew_y(wave: Wave) -> Self {
        Self {
            phase: Phase::default(),
            wave,
            manipulation: SpatialKind::SkewY as u32,
            intensity: 1.0,
            anchor: Anchor::Center.to_vec2(),
        }
    }
    pub fn with_intensity(&mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        *self
    }
    pub fn with_anchor(&mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        *self
    }

    /// Rotational degrees are converted to radians.
    pub fn from(kind: SpatialKind, unit: f32) -> Self {
        match kind {
            SpatialKind::OffsetX => Self::offset_x(Wave::constant(unit)),
            SpatialKind::OffsetY => Self::offset_y(Wave::constant(unit)),
            SpatialKind::ScaleX => Self::scale_x(Wave::constant(unit)),
            SpatialKind::ScaleY => Self::scale_y(Wave::constant(unit)),
            SpatialKind::SkewX => Self::skew_x(Wave::constant(unit)),
            SpatialKind::SkewY => Self::skew_y(Wave::constant(unit)),
            SpatialKind::Rotation => Self::rotation(Wave::constant(unit.to_radians())),
        }
    }

    /// Control when this manipulation occurs, relative to *Lifetime*
    ///
    /// start and end are fractions of *Lifetime*)
    pub fn with_phase(&mut self, start: f32, end: f32) -> Self {
        self.phase = Phase::new(start, end);
        *self
    }
}

/// Multiplier for spatial effect strength.
///
/// This is an [`EffectModifier`].
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Intensity(pub f32);

impl EffectModifier for Intensity {
    fn apply(&self, builder: &mut EffectBuilder) {
        match builder.last_effect {
            Some(LastEffect::Spatial(kind)) => {
                builder.spatial[kind].as_mut().unwrap().intensity = self.0
            }
            _ => warn!("No previous spatial-effect to modify."),
        }
    }
}

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
    kind: u32,
    freq: f32,
    amp: f32,
    bias: f32,
    phase: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    amp_envelope: Envelope,  // 32 bytes
    freq_envelope: Envelope, // 32 bytes
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
    fn new(attack: f32, hold: f32, release: f32) -> Self {
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
    fn disabled() -> Self {
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

/// Complete effect containing master timing and sub-effects.
/// RGB and Alpha are now separate for independent control.
#[repr(C)]
#[derive(Clone, Copy, Debug, ShaderType, Default)]
pub struct Effect {
    lifetime: Lifetime,
    color_effects: [ColorEffect; MAX_COLOR_FX],
    alpha_effect: AlphaEffect,
    spatial_effects: [SpatialEffect; MAX_SPATIAL_FX],
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

/// Tracks which sub-effect was most recently added to the builder.
/// ```rust
/// match builder.last_effect {
///     Some(LastEffect::Color(idx)) => builder.colors[idx],
///     Some(LastEffect::Alpha) => builder.alpha,
///     Some(LastEffect::Spatial(kind)) => builder.spatial[kind],
///     None => warn!("No previous sub-effect to modify."),
/// ```
#[derive(Clone, Copy)]
enum LastEffect {
    Color(usize),
    Alpha,
    Spatial(SpatialKind),
}

/// Builder for creating effects with chainable modifications.
///
/// All sub-effects intialize with ~
/// ```rust
/// Wave::constant(1.0)
/// ```
#[derive(Default)]
pub struct EffectBuilder {
    lifetime: Lifetime,
    colors: [Option<ColorEffect>; MAX_COLOR_FX],
    alpha: Option<AlphaEffect>,
    spatial: EnumMap<SpatialKind, Option<SpatialEffect>>, // One SpatialEffect per SpatialKind
    last_effect: Option<LastEffect>,
}

impl EffectBuilder {
    /// Start building a one-shot effect
    pub fn one_shot(now: f32, duration: f32) -> Self {
        Self {
            lifetime: Lifetime::one_shot(now, duration),
            ..default()
        }
    }

    /// Start building a looping effect
    pub fn looping(now: f32, period: f32) -> Self {
        Self {
            lifetime: Lifetime::looping(now, period),
            ..default()
        }
    }

    /// Add an RGB effect using a color that implements ColorToComponents
    ///
    /// **Important** the 4th value, usually reserved for Alpha, is repurposed as the [CompositeMode]
    ///
    /// Alpha has a dedicated building method.
    pub fn color(mut self, color: impl ColorToComponents) -> Self {
        // Find first available slot
        for (i, slot) in self.colors.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(ColorEffect::new(color.to_vec4(), Wave::constant(1.0)));
                self.last_effect = Some(LastEffect::Color(i));
                return self;
            }
        }
        warn!(
            "Maximum color effects ({}) reached, ignoring additional color",
            MAX_COLOR_FX
        );
        self
    }

    /// Add an alpha effect initialized with Wave::constant(1.0)
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = Some(AlphaEffect::new(alpha, Wave::constant(1.0)));
        self.last_effect = Some(LastEffect::Alpha);
        self
    }

    /// Shortcut  for
    /// ```
    /// alpha(0.0)
    /// ```
    /// Initialized with
    /// ```
    /// Wave::constant(1.0)
    /// ```
    /// Makes the sprite invisible.
    pub fn alpha_zero(self) -> Self {
        self.alpha(0.0)
    }

    /// Shortcut  for
    /// ```
    /// alpha(0.0)
    /// ```
    /// Initialized with
    /// ```
    /// Wave::constant(1.0)
    /// ```
    /// Makes the sprite invisible.
    pub fn invisibility(self) -> Self {
        self.alpha(0.0)
    }

    // === Spatial Effect Constructors ===

    /// Add offset_x spatial effect, intialized with
    /// ```rust
    /// Wave::constant(pixels)
    /// ```
    /// **pixels** is amplitude.
    pub fn offset_x(self, pixels: f32) -> Self {
        self.add_spatial(SpatialKind::OffsetX, pixels)
    }

    /// Add offset_y spatial effect, intialized with
    /// ```rust
    /// Wave::constant(pixels)
    /// ```
    /// **pixels** is amplitude.
    pub fn offset_y(self, pixels: f32) -> Self {
        self.add_spatial(SpatialKind::OffsetY, pixels)
    }

    /// Add scale_x spatial effect, intialized with
    /// ```rust
    /// Wave::constant(factor)
    /// ```
    /// **factor** is amplitude.
    pub fn scale_x(self, factor: f32) -> Self {
        self.add_spatial(SpatialKind::ScaleX, factor)
    }

    /// Add scale_y spatial effect, intialized with
    /// ```rust
    /// Wave::constant(factor)
    /// ```
    /// **factor** is amplitude.
    pub fn scale_y(self, factor: f32) -> Self {
        self.add_spatial(SpatialKind::ScaleY, factor)
    }

    /// Add rotation spatial effect, intialized with
    /// ```rust
    /// Wave::constant(degrees)
    /// ```
    /// **degrees** is amplitude (converted to radians).
    ///
    /// Bear in mind the conversion when modifying this.
    pub fn rotate(self, degrees: f32) -> Self {
        self.add_spatial(SpatialKind::Rotation, degrees)
    }

    /// Add skew_x spatial effect, intialized with
    /// ```rust
    /// Wave::constant(factor)
    /// ```
    /// **factor** is amplitude.
    pub fn skew_x(self, factor: f32) -> Self {
        self.add_spatial(SpatialKind::SkewX, factor)
    }

    /// Add skew_y spatial effect, intialized with
    /// ```rust
    /// Wave::constant(factor)
    /// ```
    /// **factor** is amplitude.
    pub fn skew_y(self, factor: f32) -> Self {
        self.add_spatial(SpatialKind::SkewY, factor)
    }

    /// Modify the most recent sub-effect (Color, Alpha, or Spatial) with an [`EffectModifier`]
    /// # Modifiers
    /// * **[Wave]** - *-> modifies ->* Any *note*: All fields implement [`EffectModifier`] to modify the wave, rather than replace it.
    /// *note*: `Phase` for Wave is called **[`WavePhase`]**
    /// * **[Phase]** - *-> modifies ->* Any
    /// * **[Envelope]** - *-> modifies ->* Any
    /// * **[Anchor]** - *-> modifies ->* Spatial
    /// * **[Intensity]** - *-> modifies ->* Spatial
    /// * **[BlendMode]** *-> modifies ->* Color
    /// * **[CompositeMode]** - *-> modifies ->* Color
    pub fn with(mut self, modifier: impl EffectModifier) -> Self {
        modifier.apply(&mut self);
        self
    }

    /// Consume the builder and return the constructed effect
    pub fn build(self) -> Effect {
        // 1. Create the target array filled with defaults (disabled effects)
        let mut spatial_effects = [SpatialEffect::default(); MAX_SPATIAL_FX];

        // 2. Iterate over the map values, filter out None, and fill the array
        // .flatten() removes the Options
        // .take() ensures we don't exceed the fixed array size
        for (i, effect) in self
            .spatial
            .values()
            .flatten()
            .take(MAX_SPATIAL_FX)
            .enumerate()
        {
            spatial_effects[i] = *effect;
        }

        // 3. Create the color effects array
        let mut color_effects = [ColorEffect::default(); MAX_COLOR_FX];
        for (i, color_opt) in self.colors.iter().enumerate() {
            if let Some(color) = color_opt {
                color_effects[i] = *color;
            }
        }

        Effect {
            lifetime: self.lifetime,
            color_effects,
            alpha_effect: self.alpha.unwrap_or_default(),
            spatial_effects,
        }
    }

    // === Internal Helpers ===

    fn add_spatial(mut self, kind: SpatialKind, unit_value: f32) -> Self {
        self.spatial[kind] = Some(SpatialEffect::from(kind, unit_value));
        self.last_effect = Some(LastEffect::Spatial(kind));
        self
    }
}

/// Trait that enables use of [`EffectBuilder::with()`] for modifying the most recent effect
/// in the builder chain.
///
/// You probably want this match block in `fn apply`
/// ``` rust
/// match builder.last_effect {
///     Some(LastEffect::Color(idx)) => builder.colors[idx],
///     Some(LastEffect::Alpha) => builder.alpha,
///     Some(LastEffect::Spatial(kind)) => builder.spatial[kind],
///     None => warn!("No previous sub-effect to modify."),
/// }
/// ```
pub trait EffectModifier {
    /// Modify the builder's last relevant sub-effect.
    #[doc(hidden)]
    fn apply(&self, builder: &mut EffectBuilder);
}
