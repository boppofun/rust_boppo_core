//! Obtain the current system language with [`system_language()`]
use std::{fmt::Display, ops::Deref, str::FromStr};

use std::sync::Mutex;

pub(crate) static SYSTEM_LANGUAGE: Mutex<LanguageTag> = Mutex::new(LanguageTag::US_ENGLISH);

/// Returns the current system language as a [`LanguageTag`].
///
/// The current language will never change while an activity is running.
pub fn system_language() -> LanguageTag {
    #[expect(
        clippy::missing_panics_doc,
        reason = "Only `get` and `set` ever lock the mutex, and they can't panic so the mutex can't be poisoned"
    )]
    let lock = SYSTEM_LANGUAGE.lock().unwrap();
    let value = *lock;
    drop(lock);
    value
}

/// Language tags similar to IETF BCP 47 (e.g. en-US)
///
/// For now they are defined to only support
///
/// * two character language code (e.g., en for English, zh for Chinese, es for Spanish) — based on ISO 639-1
/// * '-' hyphen character
/// * two character Region (or country) code (e.g., US for United States, CN for China, ES for Spain) — based on ISO 3166-1 alpha-2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LanguageTag {
    tag: [u8; 5],
}

impl LanguageTag {
    pub(crate) const US_ENGLISH: LanguageTag = LanguageTag::new_panics("en-US");
    pub(crate) const BR_PORTUGUESE: LanguageTag = LanguageTag::new_panics("pt-BR");

    /// Create a [`LanguageTag`] from the content of `tag`.
    /// A [`LanguageTag`] is similar to the IETF BCP 47 standard.
    ///
    /// # Errors
    ///
    /// This function will return an error if the tag does not:
    /// - have a length of 5 characters,
    /// - consist solely of ascii alphabetic characters, except that it must
    /// - have a '-' at the 2nd character position
    pub fn new(tag: impl AsRef<str>) -> Option<LanguageTag> {
        let tag = tag.as_ref();
        if tag.len() != 5 {
            return None;
        }
        if !tag
            .chars()
            .take(2)
            .chain(tag.chars().skip(3))
            .all(|c| c.is_ascii_alphabetic())
        {
            return None;
        }
        if tag.chars().nth(2) != Some('-') {
            return None;
        }

        // SAFETY: invariants ensured.
        Some(unsafe { Self::new_unchecked(tag) })
    }

    /// Returns a new [`LanguageTag`], using `tag`.
    ///
    /// It is the caller's responsibility to ensure that `tag` is a valid tag.
    ///
    /// # Panics
    ///
    /// This function will panic if the tag does not:
    /// - have a length of 5 characters,
    /// - consist solely of ascii alphabetic characters, except that it must
    /// - have a '-' at the 2nd character position
    #[must_use]
    pub(crate) const fn new_panics(tag: &str) -> LanguageTag {
        let bytes = tag.as_bytes();
        assert!(bytes.len() == 5);
        assert!(
            bytes[0].is_ascii_alphabetic()
                && bytes[1].is_ascii_alphabetic()
                && bytes[3].is_ascii_alphabetic()
                && bytes[4].is_ascii_alphabetic()
        );
        assert!(bytes[2] == b'-');

        // SAFETY: invariants asserted.
        unsafe { LanguageTag::new_unchecked(tag) }
    }

    /// Returns a new [`LanguageTag`] from `tag`.
    ///
    /// # SAFETY
    ///
    /// It is the caller's responsibility to ensure that `tag`:
    /// - has a length of 5 characters, and
    /// - consists solely of ascii-alphabetic characters.
    ///
    /// If these constraints are not met, [`LanguageTag::deref`]  (and therefore
    /// [`LanguageTag::fmt`]) is unsound.
    const unsafe fn new_unchecked(tag: &str) -> LanguageTag {
        let bytes = tag.as_bytes();
        let tag = [
            bytes[0].to_ascii_lowercase(),
            bytes[1].to_ascii_lowercase(),
            bytes[2],
            bytes[3].to_ascii_uppercase(),
            bytes[4].to_ascii_uppercase(),
        ];
        LanguageTag { tag }
    }

    /// Returns the en-US [`LanguageTag`].
    #[must_use]
    pub const fn english() -> Self {
        Self::US_ENGLISH
    }

    /// Returns the pt-BR [`LanguageTag`].
    #[must_use]
    pub const fn portuguese() -> Self {
        Self::BR_PORTUGUESE
    }
}

impl FromStr for LanguageTag {
    type Err = LanguageTagParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LanguageTag::new(s).ok_or(LanguageTagParseError)
    }
}

impl Deref for LanguageTag {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.tag is required to be valid ascii, thus valid utf8.
        unsafe { str::from_utf8_unchecked(&self.tag) }
    }
}

impl Display for LanguageTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

#[derive(Debug)]
/// Failed to parse a [`LanguageTag`] from a string.
pub struct LanguageTagParseError;

impl Display for LanguageTagParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid language tag")
    }
}

impl std::error::Error for LanguageTagParseError {}

#[cfg(test)]
mod tests {
    use super::LanguageTag;

    #[test]
    fn valid_tags_are_allowed() {
        assert!(LanguageTag::new("en-US").is_some());
        assert!(LanguageTag::new("pt-BR").is_some());
    }

    #[test]
    fn case_conversion() {
        assert_eq!(&*LanguageTag::new("en-us").unwrap(), "en-US");
        assert_eq!(&*LanguageTag::new("EN-us").unwrap(), "en-US");
        assert_eq!(&*LanguageTag::new("EN-US").unwrap(), "en-US");
        assert_eq!(&*LanguageTag::new("en-US").unwrap(), "en-US");
    }
}
