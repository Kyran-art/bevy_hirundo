use crate::internal_prelude::*;
use super::lifetime::Lifetime;
use super::color::ColorEffect;
use super::alpha::AlphaEffect;
use super::spatial::{SpatialEffect, SpatialKind};
use super::wave::Wave;
use super::effect_stack::Effect;

/// Tracks which sub-effect was most recently added to the builder.
/// ```rust
/// match builder.last_effect {
///     Some(LastEffect::Color(idx)) => builder.colors[idx],
///     Some(LastEffect::Alpha) => builder.alpha,
///     Some(LastEffect::Spatial(kind)) => builder.spatial[kind],
///     None => warn!("No previous sub-effect to modify."),
/// ```
#[derive(Clone, Copy)]
pub enum LastEffect {
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
    pub(crate) lifetime: Lifetime,
    pub(crate) colors: [Option<ColorEffect>; MAX_COLOR_FX],
    pub(crate) alpha: Option<AlphaEffect>,
    pub(crate) spatial: EnumMap<SpatialKind, Option<SpatialEffect>>, // One SpatialEffect per SpatialKind
    pub(crate) last_effect: Option<LastEffect>,
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
