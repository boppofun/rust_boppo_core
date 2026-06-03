//! Audio playback utilities.
mod controller;
mod sound_builder;
mod sound_instruction;

pub use controller::Controller;
pub use sound_builder::{ControllerOpts, SoundBuilder};
pub use sound_instruction::{ControllerParams, SoundInstruction};
