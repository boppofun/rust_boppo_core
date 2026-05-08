//! Utilities for obtaining the system language

use std::sync::Mutex;

use crate::LanguageTag;

pub(crate) static ACTIVE_LANGUAGE: Mutex<LanguageTag> = Mutex::new(LanguageTag::US_ENGLISH);

/// Returns the current system language as a [`LanguageTag`].
///
/// The current language will never change while an activity is running.
pub fn get() -> LanguageTag {
    #[expect(
        clippy::missing_panics_doc,
        reason = "Only `get` and `set` ever lock the mutex, and they can't panic so the mutex can't be poisoned"
    )]
    let lock = ACTIVE_LANGUAGE.lock().unwrap();
    let value = *lock;
    drop(lock);
    value
}
