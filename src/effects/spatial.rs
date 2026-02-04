use crate::internal_prelude::*;
use super::phase::Phase;
use super::wave::Wave;
use super::builder::{EffectBuilder, EffectModifier, LastEffect};

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
