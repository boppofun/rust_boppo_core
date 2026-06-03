//! This module is for device and framework implementers. Activity authors should not need to use any items here.
//! Much of this module is for setting up the various resources that activities will have access to.

#[cfg(feature = "wasm")]
pub mod wasm;

mod audio_paramter;
mod button_counts;

pub use audio_paramter::AudioParameter;
pub use button_counts::ButtonCounts;

use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, atomic::AtomicPtr},
};

use crate::{Lights, language::LanguageTag};

pub(super) static LIGHTS: std::sync::OnceLock<crate::main_framebuffer::MainFramebuffer> =
    std::sync::OnceLock::new();

pub type SetAndFlushLights = fn(&[crate::color::RGB; Lights::COUNT]);

/// Initializes `LIGHTS` with `L`. Should only be called once, during device initialization.
/// See [`MainFramebuffer`][crate::MainFramebuffer].
///
/// # Panics
///
/// This function will panic if it is called after `LIGHTS` has been initialized.
pub fn set_lights(lights: SetAndFlushLights) {
    LIGHTS
        .set(crate::main_framebuffer::MainFramebuffer::new(lights))
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
pub fn set_system_language(language: LanguageTag) {
    *crate::language::SYSTEM_LANGUAGE.lock().unwrap() = language;
}

type AudioControllerModifyParamsFn = fn(u64, AudioParameter, f32);

pub(super) static AUDIO_CONTROLLER_MODIFY_PARAMS_FN: std::sync::OnceLock<
    AudioControllerModifyParamsFn,
> = std::sync::OnceLock::new();

/// Initialize audio with the given `controller_modify_fn`.
pub fn init_audio(controller_modify_fn: AudioControllerModifyParamsFn) {
    let _ = AUDIO_CONTROLLER_MODIFY_PARAMS_FN.set(controller_modify_fn);

    let _ = PLAYING_SOUND_CONTROLLERS.set(RwLock::new(HashMap::new()));
}

/// An empty vec signifies that the sound is currently playing but has no controller
/// that is waiting for it to finish.
pub(crate) static PLAYING_SOUND_CONTROLLERS: OnceLock<
    RwLock<HashMap<u64, Vec<tokio::sync::oneshot::Sender<()>>>>,
> = OnceLock::new();

/// Mark the sound controller as playing.
pub fn set_sound_controller_as_playing(id: u64) {
    PLAYING_SOUND_CONTROLLERS
        .get()
        .unwrap()
        .write()
        .unwrap()
        .entry(id)
        .or_default();
}

/// Notify all waiting controllers that the sound has finished playing and remove the sound from the list of playing
pub fn on_sound_controller_finished(id: u64) {
    let mut optional_senders = {
        let mut map = PLAYING_SOUND_CONTROLLERS.get().unwrap().write().unwrap();
        map.remove(&id)
    };
    if let Some(optional_senders) = optional_senders.take() {
        for sender in optional_senders {
            let _ = sender.send(());
        }
    }
}
