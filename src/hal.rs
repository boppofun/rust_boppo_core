//! This module is for device and framework implementers. Activity authors should not need to use any items here.
//! Much of this module is for setting up the various resources that activities will have access to.

mod button_counts;

pub use button_counts::ButtonCounts;

use std::sync::atomic::AtomicPtr;

use crate::{LanguageTag, Lights};

pub(super) static LIGHTS: std::sync::OnceLock<crate::lights_setter::LightsSetter> =
    std::sync::OnceLock::new();

pub type SetAndFlushLights = fn(&[crate::color::RGB; Lights::COUNT]);

/// Initializes `LIGHTS` with `L`. Should only be called once, during device initialization.
/// See [`LightsSetter`][crate::LightsSetter].
///
/// # Panics
///
/// This function will panic if it is called after `LIGHTS` has been initialized.
pub fn set_lights(lights: SetAndFlushLights) {
    LIGHTS
        .set(crate::lights_setter::LightsSetter::new(lights))
        .unwrap();
}

pub(super) static BUTTON_EVENTS: std::sync::OnceLock<
    tokio::sync::broadcast::Sender<crate::ButtonEvent>,
> = std::sync::OnceLock::new();

/// Initializes `BUTTON_EVENTS` with `events_sender`. Should only be called once, during device initialization.
/// See [`ButtonEvents`][crate::ButtonEvents].
///
/// # Panics
/// This function will panic if it is called after `BUTTON_EVENTS` has been initialized.
pub fn set_button_events(events_sender: tokio::sync::broadcast::Sender<crate::ButtonEvent>) {
    BUTTON_EVENTS.set(events_sender).unwrap();
}

/// Global [`ButtonCounts`] receiver.
pub(super) static BUTTON_COUNTS: std::sync::OnceLock<tokio::sync::watch::Receiver<ButtonCounts>> =
    std::sync::OnceLock::new();

/// Initializes `BUTTON_COUNTS` with `rx`. Should only be called once, during device initialization.
///
/// # Panics
/// This function will panic if it is called after `BUTTON_COUNTS` has been initialized.
pub fn set_button_counts(rx: tokio::sync::watch::Receiver<ButtonCounts>) {
    BUTTON_COUNTS.set(rx).unwrap();
}

pub(super) static EXECUTOR: AtomicPtr<crate::executor::Executor> =
    AtomicPtr::new(std::ptr::null_mut());

/// Set the global executor, used for spawning new tasks.
pub fn set_executor(executor: &crate::executor::Executor) {
    EXECUTOR.store(
        std::ptr::from_ref(executor) as *mut _,
        std::sync::atomic::Ordering::SeqCst,
    );
}

/// Set the current system language to `language`.
pub fn set_active_language(language: LanguageTag) {
    let mut lock = crate::active_language::ACTIVE_LANGUAGE.lock().unwrap();
    *lock = language;
    drop(lock);
}
