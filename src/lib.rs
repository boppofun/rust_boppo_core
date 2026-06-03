#![deny(missing_docs)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
//! The core APIs for creating activities for [Boppo](https://boppo.com) shared
//! across different environments.
//!
//! ## Lights
//!
//! For setting the color of lights see the set_color and similar functions on
//! [`Button`], [`Buttons`], and [`Lights`].
//!
//! By default all changes are immediately flushed to the hardware. If you want
//! to make multiple changes in a row and performance matters you can use
//! [`Framebuffer`] or modify the auto flush behavior using [`MainFramebuffer`].
//!
//! Each Boppo button has 4 LED lights which are represented by [`LightDir`].
//!
//! For drawing and animating simple shapes treating the entire Boppo surface as
//! a display see [`lights_plane`].
//!
//! ## Button Events
//!
//! You can receive button change events as an async stream using
//! [`ButtonEvents`].
//!
//! You can also query the current state of the buttons using
//! [`Button::is_pressed`] and [`Buttons::currently_pressed`].
//!
//! You can use [`Button::wait_for_press`] and [`Button::wait_for_release`] to
//! wait for a button to be in a specific state.
//!
//! ## Audio
//!
//! Provides [`SoundInstruction`] and [`SoundBuilder`] for creating sounds.
//! Playing those sounds is environment-dependent.
//!
//! ## Guidelines
//!
//! See Boppo's
//! [Activity Guidelines](https://developer.boppo.com/docs/activity-guidelines)
//! for guidelines on creating great activities.

pub mod audio;
pub mod color;
pub mod executor;
pub mod internal;
pub mod interpolation;
pub mod language;
pub mod lights_plane;
pub mod update_loop;

mod button;
mod button_events;
mod buttons;
mod easings;
mod framebuffer;
mod lights;
mod main_framebuffer;
mod short_duration;

pub use button::{Button, Column, Row};
pub use button_events::ButtonEvent;
pub use button_events::ButtonEvents;
pub use buttons::Buttons;
pub use easings::Easing;
pub use framebuffer::Framebuffer;
pub use lights::LightDir;
pub use lights::Lights;
pub use log;
pub use main_framebuffer::MainFramebuffer;
pub use short_duration::ShortDuration;
