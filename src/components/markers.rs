use crate::internal_prelude::*;

/// Marker component to delay visibility, veiling ghost sprites during initialization.
#[derive(Component, Default)]
pub struct VfxGhostBuffer;

/// Component marker for entities using broadcast material
#[derive(Component)]
pub struct VfxBroadcast;
