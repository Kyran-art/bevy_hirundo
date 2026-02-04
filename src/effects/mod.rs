// Effect sub-modules
mod lifetime;
mod phase;
mod color;
mod alpha;
mod spatial;
mod wave;
mod envelope;
mod effect_stack;
mod builder;

// Re-export all public types
pub use lifetime::*;
pub use phase::*;
pub use color::*;
pub use alpha::*;
pub use spatial::*;
pub use wave::*;
pub use envelope::*;
pub use effect_stack::*;
pub use builder::*;
