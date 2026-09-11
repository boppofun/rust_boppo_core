use crate::language::{LanguageTag, system_language};
use std::{borrow::Cow, path::PathBuf, str::FromStr};

/// Configuration parameters for [`SoundInstruction::SpeakNumber`].
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SpeakNumberConfig {
    /// The path prefix where files will be searched. Defaults to `numbers`.
    /// NOTE: It's important that the files you use are formatted the same as the files in `numbers/en-US`.
    pub prefix: PathBuf,
    /// The file extension of the expected audio files. Defaults to `qoa`.
    pub extension: Cow<'static, str>,
    /// The language to use for conversion. Defaults to the active device language.
    pub language: LanguageTag,
}

impl Default for SpeakNumberConfig {
    fn default() -> Self {
        Self {
            prefix: PathBuf::from_str("numbers").unwrap(),
            extension: Cow::Borrowed("qoa"),
            language: system_language(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::audio::SpeakNumberConfig;

    #[test]
    fn config_deserialize() {
        assert_eq!(
            SpeakNumberConfig::default(),
            serde_json::from_str(
                r#"{"prefix": "numbers", "extension": "qoa", "language": "en-US"}"#
            )
            .expect("should be able to deserialize")
        );
    }
}
