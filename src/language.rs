use std::{fmt::Display, ops::Deref, str::FromStr};

use anyhow::ensure;

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
    pub fn new(tag: impl AsRef<str>) -> anyhow::Result<LanguageTag> {
        let tag = tag.as_ref();
        ensure!(tag.len() == 5);
        ensure!(
            tag.chars()
                .take(2)
                .chain(tag.chars().skip(3))
                .all(|c| c.is_ascii_alphabetic())
        );
        ensure!(tag.chars().nth(2) == Some('-'));

        // SAFETY: invariants ensured.
        Ok(unsafe { Self::new_unchecked(tag) })
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LanguageTag::new(s)
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

#[cfg(test)]
mod tests {
    use crate::LanguageTag;

    #[test]
    fn valid_tags_are_allowed() {
        assert!(LanguageTag::new("en-US").is_ok());
        assert!(LanguageTag::new("pt-BR").is_ok());
    }

    #[test]
    fn case_conversion() {
        assert_eq!(&*LanguageTag::new("en-us").unwrap(), "en-US");
        assert_eq!(&*LanguageTag::new("EN-us").unwrap(), "en-US");
        assert_eq!(&*LanguageTag::new("EN-US").unwrap(), "en-US");
        assert_eq!(&*LanguageTag::new("en-US").unwrap(), "en-US");
    }
}
