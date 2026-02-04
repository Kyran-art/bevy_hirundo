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
